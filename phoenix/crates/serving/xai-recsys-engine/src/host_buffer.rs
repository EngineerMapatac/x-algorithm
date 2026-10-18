// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::io;
use std::ptr::NonNull;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;

use crate::mem_util::{madvise_hugepage_internal, mbind_interleave_internal};

pub const AVX_512_ALIGNMENT: usize = 64;

#[cfg(target_arch = "aarch64")]
const PAGE_SIZE: usize = 2usize.pow(16);
#[cfg(not(target_arch = "aarch64"))]
const PAGE_SIZE: usize = 2usize.pow(12);

const DEFAULT_INIT_THREADS: usize = 128;
const HUGEPAGE_MIN_BYTES: usize = 1 << 21;

pub struct AlignedHostBuffer {
    base: NonNull<u8>,
    layout: Layout,
    data: NonNull<u8>,
    len: usize,
}

unsafe impl Send for AlignedHostBuffer {}
unsafe impl Sync for AlignedHostBuffer {}

impl AlignedHostBuffer {
    pub fn allocate(nbytes: usize) -> io::Result<Self> {
        Self::allocate_with(nbytes, false)
    }

    fn allocate_with(nbytes: usize, interleave: bool) -> io::Result<Self> {
        if nbytes == 0 {
            let layout = Layout::from_size_align(AVX_512_ALIGNMENT, AVX_512_ALIGNMENT)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            let base = unsafe { alloc_zeroed(layout) };
            let base = NonNull::new(base).ok_or(io::Error::from(io::ErrorKind::OutOfMemory))?;
            return Ok(Self {
                base,
                layout,
                data: base,
                len: 0,
            });
        }

        let alloc_size = nbytes
            .checked_add(AVX_512_ALIGNMENT)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "size overflow"))?;
        let layout = Layout::from_size_align(alloc_size, AVX_512_ALIGNMENT)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let base_ptr = unsafe { alloc_zeroed(layout) };
        let base = NonNull::new(base_ptr).ok_or(io::Error::from(io::ErrorKind::OutOfMemory))?;

        let addr = base.as_ptr() as usize;
        let aligned = (addr + AVX_512_ALIGNMENT - 1) & !(AVX_512_ALIGNMENT - 1);
        let data = unsafe { NonNull::new_unchecked(aligned as *mut u8) };

        let mut buf = Self {
            base,
            layout,
            data,
            len: nbytes,
        };

        if nbytes >= HUGEPAGE_MIN_BYTES {
            let _ = madvise_hugepage_internal(buf.as_mut_slice());
            if interleave && let Err(e) = mbind_interleave_internal(buf.as_mut_slice()) {
                log::warn!("host buffer ({nbytes} bytes): NUMA interleave failed: {e}");
            }
        }
        fault_in_pages(buf.as_mut_slice(), DEFAULT_INIT_THREADS);

        Ok(buf)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_ptr()
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.len) }
    }
}

impl Drop for AlignedHostBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.base.as_ptr(), self.layout);
        }
    }
}

pub fn allocate_emb_table(
    height: usize,
    width: usize,
    itemsize: usize,
) -> io::Result<AlignedHostBuffer> {
    if itemsize == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "itemsize must be > 0",
        ));
    }
    let nbytes = height
        .checked_mul(width)
        .and_then(|x| x.checked_mul(itemsize))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "emb_table size overflow"))?;
    AlignedHostBuffer::allocate_with(nbytes, true)
}

fn fault_in_pages(slice: &mut [u8], num_threads: usize) {
    if slice.is_empty() || num_threads == 0 {
        return;
    }
    let n = num_threads.max(1);
    let chunk_size = slice.len().div_ceil(n);
    slice.par_chunks_mut(chunk_size).for_each(|chunk| {
        chunk
            .iter_mut()
            .step_by(PAGE_SIZE)
            .for_each(|elem| *elem = 0);
    });
}

#[pyclass(name = "EmbTableHandle", module = "xai_recsys_engine")]
pub struct EmbTableHandle {
    buf: AlignedHostBuffer,
    height: usize,
    width: usize,
    itemsize: usize,
}

#[pymethods]
impl EmbTableHandle {
    #[new]
    #[pyo3(signature = (height, width, itemsize))]
    fn new(height: usize, width: usize, itemsize: usize) -> PyResult<Self> {
        let buf = allocate_emb_table(height, width, itemsize)
            .map_err(|e| PyValueError::new_err(format!("allocate_emb_table failed: {e}")))?;
        Ok(Self {
            buf,
            height,
            width,
            itemsize,
        })
    }

    #[getter]
    fn address(&self) -> usize {
        self.buf.as_ptr() as usize
    }

    #[getter]
    fn nbytes(&self) -> usize {
        self.buf.len()
    }

    #[getter]
    fn height(&self) -> usize {
        self.height
    }

    #[getter]
    fn width(&self) -> usize {
        self.width
    }

    #[getter]
    fn itemsize(&self) -> usize {
        self.itemsize
    }

    #[getter]
    fn shape(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    fn __repr__(&self) -> String {
        format!(
            "EmbTableHandle(shape=({}, {}), itemsize={}, nbytes={}, address={:#x})",
            self.height,
            self.width,
            self.itemsize,
            self.buf.len(),
            self.buf.as_ptr() as usize
        )
    }
}

impl EmbTableHandle {
    pub fn into_buffer(self) -> AlignedHostBuffer {
        self.buf
    }

    pub fn buffer(&self) -> &AlignedHostBuffer {
        &self.buf
    }

    pub fn buffer_mut(&mut self) -> &mut AlignedHostBuffer {
        &mut self.buf
    }
}

#[pyfunction]
#[pyo3(name = "allocate_emb_table", signature = (height, width, itemsize))]
pub fn allocate_emb_table_py(
    height: usize,
    width: usize,
    itemsize: usize,
) -> PyResult<(usize, usize, EmbTableHandle)> {
    let handle = EmbTableHandle::new(height, width, itemsize)?;
    let addr = handle.address();
    let nbytes = handle.nbytes();
    Ok((addr, nbytes, handle))
}
