// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use futures::future::join_all;
use std::{cmp, ffi::c_int, io, mem};
use tokio::task;

#[cfg(all(test, target_os = "linux"))]
use nix::libc::MPOL_DEFAULT;
#[cfg(target_os = "linux")]
use nix::libc::{
    MADV_HUGEPAGE, MADV_NORMAL, MADV_SEQUENTIAL, MPOL_BIND, MPOL_INTERLEAVE, MPOL_LOCAL,
    SYS_get_mempolicy, SYS_mbind, SYS_set_mempolicy, madvise, syscall,
};

pub const NODE_MASK_SIZE: usize = 16;
const MPOL_F_MEMS_ALLOWED: u64 = 4;

pub fn get_numa_policy(flags: u64) -> Result<(c_int, [u64; NODE_MASK_SIZE]), io::Error> {
    let _used = (flags, MPOL_F_MEMS_ALLOWED);

    #[cfg(target_os = "linux")]
    {
        let mut mode: c_int = 0;
        let mut node_mask = [0u64; NODE_MASK_SIZE];
        let ret = unsafe {
            syscall(
                SYS_get_mempolicy,
                &mut mode,
                node_mask.as_mut_ptr(),
                NODE_MASK_SIZE as u64 * 64,
                0usize,
                flags,
            )
        };
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((mode, node_mask))
    }
    #[cfg(not(target_os = "linux"))]
    Ok((0, [0; NODE_MASK_SIZE]))
}

pub fn get_numa_node_count() -> Result<u32, io::Error> {
    #[cfg(target_os = "linux")]
    {
        let mut count = 0;
        for mask in get_numa_policy(MPOL_F_MEMS_ALLOWED)?.1 {
            if mask != u64::MAX {
                count += mask.trailing_ones();
                break;
            }
            count += 64;
        }
        Ok(count)
    }
    #[cfg(not(target_os = "linux"))]
    Ok(1)
}

pub fn set_numa_policy(mode: c_int, node_mask: [u64; NODE_MASK_SIZE]) -> Result<(), io::Error> {
    let _used = (mode, &node_mask);

    #[cfg(target_os = "linux")]
    {
        let ret = unsafe {
            syscall(
                SYS_set_mempolicy,
                mode,
                node_mask.as_ptr(),
                NODE_MASK_SIZE * 64,
            )
        };
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn opt_out_of_numa_balancing() -> Result<(), io::Error> {
    #[cfg(target_os = "linux")]
    return set_numa_policy(MPOL_LOCAL, [0; NODE_MASK_SIZE]);
    #[cfg(not(target_os = "linux"))]
    Ok(())
}

#[cfg(target_arch = "aarch64")]
const HUGE_PAGE_SIZE: usize = 2usize.pow(29);
#[cfg(not(target_arch = "aarch64"))]
const HUGE_PAGE_SIZE: usize = 2usize.pow(21);

pub fn madvise_hugepage_internal(slice: &mut [u8]) -> Result<(), io::Error> {
    let _used = (&slice, HUGE_PAGE_SIZE);

    #[cfg(target_os = "linux")]
    unsafe {
        use std::ffi::c_void;

        let offset = (!(slice.as_ptr() as usize) + 1) & (HUGE_PAGE_SIZE - 1);
        let len = slice.len().saturating_sub(offset) & !(HUGE_PAGE_SIZE - 1);
        if len == 0 {
            return Ok(());
        }
        let ret = madvise(
            slice.as_mut_ptr().add(offset) as *mut c_void,
            len,
            MADV_HUGEPAGE,
        );
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

pub fn mbind_interleave_internal(slice: &mut [u8]) -> Result<(), io::Error> {
    let _used = (&slice, HUGE_PAGE_SIZE);

    #[cfg(target_os = "linux")]
    unsafe {
        let offset = (!(slice.as_ptr() as usize) + 1) & (HUGE_PAGE_SIZE - 1);
        let len = slice.len().saturating_sub(offset) & !(HUGE_PAGE_SIZE - 1);
        if len == 0 {
            return Ok(());
        }
        let nodes = interleave_nodes()?;
        if nodes.iter().map(|m| m.count_ones()).sum::<u32>() < 2 {
            return Ok(());
        }
        let ret = syscall(
            SYS_mbind,
            slice.as_mut_ptr().add(offset),
            len,
            MPOL_INTERLEAVE,
            nodes.as_ptr(),
            NODE_MASK_SIZE * 64,
            0u32,
        );
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn interleave_nodes() -> Result<[u64; NODE_MASK_SIZE], io::Error> {
    let mut nodes = get_numa_policy(MPOL_F_MEMS_ALLOWED)?.1;
    if let Some(with_cpus) = std::fs::read_to_string("/sys/devices/system/node/has_cpu")
        .ok()
        .and_then(|text| parse_node_list(&text))
    {
        for (node, cpu) in nodes.iter_mut().zip(with_cpus) {
            *node &= cpu;
        }
    }
    Ok(nodes)
}

#[cfg(target_os = "linux")]
fn parse_node_list(text: &str) -> Option<[u64; NODE_MASK_SIZE]> {
    let mut mask = [0u64; NODE_MASK_SIZE];
    for part in text.trim().split(',').filter(|part| !part.is_empty()) {
        let (lo, hi) = match part.split_once('-') {
            Some((lo, hi)) => (lo.parse::<usize>().ok()?, hi.parse::<usize>().ok()?),
            None => {
                let node = part.parse::<usize>().ok()?;
                (node, node)
            }
        };
        if lo > hi || hi >= NODE_MASK_SIZE * 64 {
            return None;
        }
        for node in lo..=hi {
            mask[node / 64] |= 1 << (node % 64);
        }
    }
    Some(mask)
}

pub fn madvise_sequential_internal(slice: &mut [u8]) -> Result<(), io::Error> {
    #[cfg(target_os = "linux")]
    unsafe {
        use std::ffi::c_void;
        let ret = madvise(
            slice.as_mut_ptr() as *mut c_void,
            slice.len(),
            MADV_SEQUENTIAL,
        );
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    let _ = slice;
    Ok(())
}

pub fn madvise_normal_internal(slice: &mut [u8]) -> Result<(), io::Error> {
    #[cfg(target_os = "linux")]
    unsafe {
        use std::ffi::c_void;
        let ret = madvise(slice.as_mut_ptr() as *mut c_void, slice.len(), MADV_NORMAL);
        if ret != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    let _ = slice;
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn copy_nontemporal(dst: &mut [u8], src: &[u8]) {
    use std::arch::x86_64::{__m256i, _mm256_loadu_si256, _mm256_stream_si256};

    const NT_THRESHOLD: usize = 4096;
    const CHUNK: usize = 32;

    assert_eq!(dst.len(), src.len());
    let len = dst.len();

    if len < NT_THRESHOLD {
        dst.copy_from_slice(src);
        return;
    }

    let dst_ptr = dst.as_mut_ptr();
    let align_offset = dst_ptr.align_offset(CHUNK).min(len);

    dst[..align_offset].copy_from_slice(&src[..align_offset]);

    let middle_end = align_offset + ((len - align_offset) / CHUNK) * CHUNK;
    let mut i = align_offset;
    while i < middle_end {
        unsafe {
            let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
            _mm256_stream_si256(dst_ptr.add(i) as *mut __m256i, s);
        }
        i += CHUNK;
    }

    dst[middle_end..].copy_from_slice(&src[middle_end..]);
}

#[cfg(not(target_arch = "x86_64"))]
pub fn copy_nontemporal(dst: &mut [u8], src: &[u8]) {
    dst.copy_from_slice(src);
}

#[derive(Copy, Clone, Debug)]
pub enum InitKind {
    Striped(usize),
    Interleaved,
    Default,
}

pub async fn init_mem(buf: &mut [u8], kind: InitKind) -> Result<(), io::Error> {
    madvise_hugepage_internal(buf)?;
    let mut buf: &'static mut [u8] = unsafe { mem::transmute(buf) };
    let (n, stripe_size) = if let InitKind::Striped(size) = kind {
        (get_numa_node_count()? as usize, size)
    } else {
        (1, usize::MAX)
    };
    let step = buf.len() / (n << 10);
    let mut bufz = Vec::with_capacity(n);
    for _ in 0..n {
        bufz.push(Vec::new());
    }
    let mut pos = 0usize;
    let mut i = n - 1;
    while !buf.is_empty() {
        let mut delta = pos.next_multiple_of(stripe_size) - pos;
        if delta == 0 {
            delta = stripe_size;
            i = (i + 1) % n;
        }
        let (buf0, buf1) = buf.split_at_mut(cmp::min(step, cmp::min(delta, buf.len())));
        pos += buf0.len();
        bufz[i].push(buf0);
        buf = buf1;
    }
    for (i, bufs) in bufz.into_iter().enumerate() {
        let _used = i;
        join_all(bufs.into_iter().map(|buf| {
            task::spawn(async move {
                let (mode, node_mask) = get_numa_policy(0)?;
                #[cfg(target_os = "linux")]
                match kind {
                    InitKind::Striped(_) => {
                        let mut node_mask = [0; NODE_MASK_SIZE];
                        node_mask[i / 64] = 1u64 << (i % 64);
                        set_numa_policy(MPOL_BIND, node_mask)?;
                    }
                    InitKind::Interleaved => {
                        set_numa_policy(MPOL_INTERLEAVE, get_numa_policy(MPOL_F_MEMS_ALLOWED)?.1)?;
                    }
                    InitKind::Default => {}
                }
                for i in (0..buf.len()).step_by(1 << 11) {
                    buf[i] = 0;
                }
                set_numa_policy(mode, node_mask)
            })
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Result<Vec<()>, _>>()?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    const MPOL_F_ADDR: u64 = 2;

    fn policy_at(addr: *const u8) -> c_int {
        let mut mode: c_int = -1;
        let ret = unsafe {
            syscall(
                SYS_get_mempolicy,
                &mut mode,
                std::ptr::null_mut::<u64>(),
                0usize,
                addr,
                MPOL_F_ADDR,
            )
        };
        assert_eq!(ret, 0, "get_mempolicy: {}", io::Error::last_os_error());
        mode
    }

    #[test]
    fn numa_balancing_opt_out_is_inherited_by_later_threads() {
        std::thread::spawn(|| {
            match opt_out_of_numa_balancing() {
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return,
                r => r.unwrap(),
            }
            assert_eq!(get_numa_policy(0).unwrap().0, MPOL_LOCAL);
            let child = std::thread::spawn(|| get_numa_policy(0).unwrap().0);
            assert_eq!(child.join().unwrap(), MPOL_LOCAL);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn mbind_interleave_covers_the_whole_huge_pages() {
        let mut buf = vec![0u8; 3 * HUGE_PAGE_SIZE];
        match mbind_interleave_internal(&mut buf) {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => return,
            r => r.unwrap(),
        }
        let nodes = interleave_nodes().unwrap();
        let want = if nodes.iter().map(|m| m.count_ones()).sum::<u32>() >= 2 {
            MPOL_INTERLEAVE
        } else {
            MPOL_DEFAULT
        };
        let offset = (!(buf.as_ptr() as usize) + 1) & (HUGE_PAGE_SIZE - 1);
        let first = unsafe { buf.as_ptr().add(offset) };
        assert_eq!(policy_at(first), want);
        assert_eq!(
            policy_at(unsafe { first.add(2 * HUGE_PAGE_SIZE - 1) }),
            want
        );
    }

    #[test]
    fn interleave_nodes_have_cpus_and_are_allowed() {
        let allowed = get_numa_policy(MPOL_F_MEMS_ALLOWED).unwrap().1;
        let nodes = interleave_nodes().unwrap();
        for (node, allowed) in nodes.iter().zip(allowed) {
            assert_eq!(node & !allowed, 0);
        }
        let Ok(text) = std::fs::read_to_string("/sys/devices/system/node/has_cpu") else {
            return;
        };
        let with_cpus = parse_node_list(&text).unwrap();
        for (node, cpu) in nodes.iter().zip(with_cpus) {
            assert_eq!(node & !cpu, 0);
        }
    }

    #[test]
    fn node_lists_parse_like_sysfs() {
        let mask = |nodes: &[usize]| {
            let mut mask = [0u64; NODE_MASK_SIZE];
            for &n in nodes {
                mask[n / 64] |= 1 << (n % 64);
            }
            mask
        };
        assert_eq!(parse_node_list("0-1\n"), Some(mask(&[0, 1])));
        assert_eq!(parse_node_list("0,2-3,70"), Some(mask(&[0, 2, 3, 70])));
        assert_eq!(parse_node_list("\n"), Some(mask(&[])));
        assert_eq!(parse_node_list("1023"), Some(mask(&[1023])));
        for bad in ["1-0", "a", "0-", "1024"] {
            assert_eq!(parse_node_list(bad), None, "{bad}");
        }
    }
}
