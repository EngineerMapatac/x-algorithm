// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use half::f16;
use memmap2::{MmapMut, MmapOptions};

use crate::snowflake;

pub struct EmbeddingCache {
    inner: RwLock<Inner>,
    ttl: Duration,
}

struct Slot {
    post_id: u64,
    embedding: Arc<Vec<f16>>,
}

struct Inner {
    slots: Vec<Option<Slot>>,
    index: BTreeMap<u64, usize>,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl EmbeddingCache {
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);
        Self {
            inner: RwLock::new(Inner {
                slots,
                index: BTreeMap::new(),
                head: 0,
                tail: 0,
                capacity,
            }),
            ttl,
        }
    }

    pub fn insert(&self, post_id: u64, embedding: Arc<Vec<f16>>) {
        let mut inner = self.inner.write().unwrap();
        Self::insert_inner(&mut inner, post_id, embedding);
    }

    pub fn bulk_insert(&self, records: impl Iterator<Item = (u64, Arc<Vec<f16>>)>) {
        let mut inner = self.inner.write().unwrap();
        for (post_id, embedding) in records {
            Self::insert_inner(&mut inner, post_id, embedding);
        }
        while inner.tail != inner.head {
            if let Some(slot) = &inner.slots[inner.tail]
                && snowflake::snowflake_age(slot.post_id).is_some_and(|age| age > self.ttl)
            {
                let tail = inner.tail;
                let old = inner.slots[tail].take().unwrap();
                inner.index.remove(&old.post_id);
                inner.tail = (inner.tail + 1) % inner.capacity;
                continue;
            }
            break;
        }
    }

    fn insert_inner(inner: &mut Inner, post_id: u64, embedding: Arc<Vec<f16>>) {
        if let Some(&slot_idx) = inner.index.get(&post_id) {
            inner.slots[slot_idx] = Some(Slot { post_id, embedding });
            return;
        }

        if inner.index.len() >= inner.capacity {
            if let Some(old) = inner.slots[inner.tail].take() {
                inner.index.remove(&old.post_id);
            }
            inner.tail = (inner.tail + 1) % inner.capacity;
        }

        let slot_idx = inner.head;
        inner.slots[slot_idx] = Some(Slot { post_id, embedding });
        inner.index.insert(post_id, slot_idx);
        inner.head = (inner.head + 1) % inner.capacity;
    }

    pub fn get(&self, post_id: u64) -> Option<Arc<Vec<f16>>> {
        if snowflake::snowflake_age(post_id).is_some_and(|age| age > self.ttl) {
            return None;
        }
        let inner = self.inner.read().unwrap();
        let &slot_idx = inner.index.get(&post_id)?;
        inner.slots[slot_idx]
            .as_ref()
            .map(|s| Arc::clone(&s.embedding))
    }

    pub fn copy_to(&self, post_id: u64, dst: &mut [f16]) -> bool {
        if snowflake::snowflake_age(post_id).is_some_and(|age| age > self.ttl) {
            return false;
        }
        let inner = self.inner.read().unwrap();
        if let Some(&slot_idx) = inner.index.get(&post_id)
            && let Some(slot) = &inner.slots[slot_idx]
        {
            assert_eq!(
                dst.len(),
                slot.embedding.len(),
                "dst length must match embedding dim"
            );
            dst.copy_from_slice(&slot.embedding);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn drain_all(&self) -> Vec<(u64, Vec<f16>)> {
        let mut inner = self.inner.write().unwrap();
        let indices: Vec<(u64, usize)> = inner.index.iter().map(|(&k, &v)| (k, v)).collect();
        let mut out = Vec::with_capacity(indices.len());
        for (post_id, slot_idx) in indices {
            if let Some(slot) = inner.slots[slot_idx].take() {
                out.push((
                    post_id,
                    Arc::try_unwrap(slot.embedding).unwrap_or_else(|arc| (*arc).clone()),
                ));
            }
        }
        inner.index.clear();
        inner.head = 0;
        inner.tail = 0;
        out
    }
}

const HEADER_SIZE: usize = 64;

const SLOT_HEADER_SIZE: usize = 8;

const LMDB_META_KEY_EMB_DIM: u64 = 0;

pub struct LmdbEmbeddingCache {
    env: heed::Env,
    db: heed::Database<
        heed::types::U64<heed::byteorder::NativeEndian>,
        heed::types::U64<heed::byteorder::NativeEndian>,
    >,
    mmap: MmapMut,
    capacity: usize,
    emb_dim: usize,
    slot_size: usize,
    ttl: std::time::Duration,
}

unsafe impl Send for EmbeddingCache {}
unsafe impl Sync for EmbeddingCache {}

#[repr(C)]
struct Header {
    count: AtomicU64,
    head: AtomicU64,
    tail: AtomicU64,
}

impl LmdbEmbeddingCache {
    pub fn create(
        lmdb_path: &Path,
        data_path: &Path,
        capacity: usize,
        emb_dim: usize,
        ttl: std::time::Duration,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(capacity > 0 && emb_dim > 0);

        std::fs::create_dir_all(lmdb_path)?;
        let index_map_size = (capacity * 1024).max(64 * 1024 * 1024);
        const PAGE_ALIGN: usize = 65536;
        let index_map_size = index_map_size.div_ceil(PAGE_ALIGN) * PAGE_ALIGN;

        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(index_map_size)
                .max_readers(1024)
                .open(lmdb_path)?
        };
        let mut wtxn = env.write_txn()?;
        let db = env.create_database(&mut wtxn, None)?;
        db.put(&mut wtxn, &LMDB_META_KEY_EMB_DIM, &(emb_dim as u64))?;
        wtxn.commit()?;

        let slot_size = SLOT_HEADER_SIZE + emb_dim * std::mem::size_of::<f16>();
        let total_size = HEADER_SIZE + capacity * slot_size;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(data_path)?;
        file.set_len(total_size as u64)?;
        let mmap = unsafe { MmapOptions::new().len(total_size).map_mut(&file)? };

        #[cfg(target_os = "linux")]
        {
            use std::ffi::c_void;
            unsafe extern "C" {
                fn madvise(addr: *mut c_void, len: usize, advice: i32) -> i32;
            }
            const MADV_POPULATE_WRITE: i32 = 23;
            let ret = unsafe {
                madvise(
                    mmap.as_ptr() as *mut c_void,
                    mmap.len(),
                    MADV_POPULATE_WRITE,
                )
            };
            if ret != 0 {
                log::warn!("madvise(POPULATE_WRITE) failed, falling back to manual pre-fault");
                let ptr = mmap.as_ptr() as *mut u8;
                for offset in (0..mmap.len()).step_by(4096) {
                    unsafe { std::ptr::write_volatile(ptr.add(offset), 0) };
                }
            }
        }

        log::info!(
            "EmbeddingCache: created lmdb={} data={} (capacity={}, emb_dim={}, \
             index={:.2} GB, data={:.2} GB)",
            lmdb_path.display(),
            data_path.display(),
            capacity,
            emb_dim,
            index_map_size as f64 / 1e9,
            total_size as f64 / 1e9,
        );

        Ok(Self {
            env,
            db,
            mmap,
            capacity,
            emb_dim,
            slot_size,
            ttl,
        })
    }

    pub fn open(
        lmdb_path: &Path,
        data_path: &Path,
        ttl: std::time::Duration,
    ) -> anyhow::Result<Self> {
        let data_mdb = lmdb_path.join("data.mdb");
        let map_size = std::fs::metadata(&data_mdb)
            .map(|m| m.len() as usize)
            .unwrap_or(0)
            .max(1024 * 1024);

        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(map_size)
                .max_readers(1024)
                .open(lmdb_path)?
        };
        let rtxn = env.read_txn()?;
        let db = env.open_database(&rtxn, None)?.ok_or_else(|| {
            anyhow::anyhow!("EmbeddingCache: no database in {}", lmdb_path.display())
        })?;
        let emb_dim = db
            .get(&rtxn, &LMDB_META_KEY_EMB_DIM)?
            .ok_or_else(|| anyhow::anyhow!("EmbeddingCache: missing emb_dim metadata"))?
            as usize;
        rtxn.commit()?;

        let file = OpenOptions::new().read(true).write(true).open(data_path)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        let slot_size = SLOT_HEADER_SIZE + emb_dim * std::mem::size_of::<f16>();
        let capacity = (mmap.len() - HEADER_SIZE) / slot_size;

        log::info!(
            "EmbeddingCache: opened lmdb={} data={} (capacity={}, emb_dim={})",
            lmdb_path.display(),
            data_path.display(),
            capacity,
            emb_dim,
        );

        Ok(Self {
            env,
            db,
            mmap,
            capacity,
            emb_dim,
            slot_size,
            ttl,
        })
    }

    pub fn insert_many(&self, records: Vec<(u64, Vec<f16>)>) {
        let mut wtxn = match self.env.write_txn() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("LmdbEmbeddingCache::insert_many: write txn failed: {}", e);
                log::error!("LmdbEmbeddingCache::insert_many: write txn failed: {}", e);
                return;
            }
        };

        let header = self.header();
        let mut head = header.head.load(Ordering::Relaxed) as usize;
        let mut tail = header.tail.load(Ordering::Relaxed) as usize;
        let mut count = header.count.load(Ordering::Relaxed) as usize;

        for (post_id, embedding) in records {
            if post_id == 0 || embedding.len() != self.emb_dim {
                continue;
            }

            if let Ok(Some(existing_idx)) = self.db.get(&wtxn, &post_id) {
                let idx = existing_idx as usize;
                self.write_slot_post_id(idx, 0, Ordering::Release);
                self.write_slot_embedding(idx, &embedding);
                self.write_slot_post_id(idx, post_id, Ordering::Release);
                continue;
            }

            let slot_idx;
            if count >= self.capacity {
                slot_idx = tail;
                let victim = self.read_slot_post_id(tail, Ordering::Relaxed);
                self.write_slot_post_id(tail, 0, Ordering::Release);
                if victim != 0 {
                    let _ = self.db.delete(&mut wtxn, &victim);
                }
                tail = (tail + 1) % self.capacity;
                head = (head + 1) % self.capacity;
            } else {
                slot_idx = head;
                head = (head + 1) % self.capacity;
                count += 1;
            }

            self.write_slot_embedding(slot_idx, &embedding);
            self.write_slot_post_id(slot_idx, post_id, Ordering::Release);

            if let Err(e) = self.db.put(&mut wtxn, &post_id, &(slot_idx as u64)) {
                eprintln!("LmdbEmbeddingCache: put failed: {}", e);
                log::error!("LmdbEmbeddingCache: put failed: {}", e);
                return;
            }
        }

        while count > 0 {
            let pid = self.read_slot_post_id(tail, Ordering::Relaxed);
            if pid == 0 {
                break;
            }
            match snowflake::snowflake_age(pid) {
                Some(age) if age > self.ttl => {
                    self.write_slot_post_id(tail, 0, Ordering::Release);
                    let _ = self.db.delete(&mut wtxn, &pid);
                    tail = (tail + 1) % self.capacity;
                    count -= 1;
                }
                _ => break,
            }
        }

        if let Err(e) = wtxn.commit() {
            eprintln!("LmdbEmbeddingCache::insert_many: commit failed: {}", e);
            log::error!("LmdbEmbeddingCache::insert_many: commit failed: {}", e);
            return;
        }

        header.head.store(head as u64, Ordering::Release);
        header.tail.store(tail as u64, Ordering::Release);
        header.count.store(count as u64, Ordering::Release);
    }

    pub fn copy_to(&self, post_id: u64, dst: &mut [f16]) -> bool {
        if post_id == 0 {
            return false;
        }
        if snowflake::snowflake_age(post_id).is_some_and(|age| age > self.ttl) {
            return false;
        }
        let rtxn = match self.env.read_txn() {
            Ok(t) => t,
            Err(e) => {
                log::error!("LmdbEmbeddingCache::copy_to: read_txn failed: {}", e);
                return false;
            }
        };
        match self.db.get(&rtxn, &post_id) {
            Ok(Some(slot_idx)) => {
                let idx = slot_idx as usize;
                if self.read_slot_post_id(idx, Ordering::Acquire) != post_id {
                    return false;
                }
                self.read_slot_embedding(idx, dst);
                true
            }
            _ => false,
        }
    }

    pub fn copy_batch(&self, post_ids: &[u64], emb_dim: usize, out: &mut [f16]) -> usize {
        let rtxn = match self.env.read_txn() {
            Ok(t) => t,
            Err(e) => {
                log::error!("LmdbEmbeddingCache::copy_batch: read_txn failed: {}", e);
                return post_ids.len();
            }
        };
        let mut missing = 0usize;
        for (idx, &post_id) in post_ids.iter().enumerate() {
            if post_id == 0 || snowflake::snowflake_age(post_id).is_some_and(|age| age > self.ttl) {
                missing += 1;
                continue;
            }
            let dst = &mut out[idx * emb_dim..(idx + 1) * emb_dim];
            match self.db.get(&rtxn, &post_id) {
                Ok(Some(slot_idx)) => {
                    let si = slot_idx as usize;
                    if self.read_slot_post_id(si, Ordering::Acquire) != post_id {
                        missing += 1;
                    } else {
                        self.read_slot_embedding(si, dst);
                    }
                }
                _ => missing += 1,
            }
        }
        missing
    }

    pub fn len(&self) -> usize {
        self.header().count.load(Ordering::Relaxed) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn emb_dim(&self) -> usize {
        self.emb_dim
    }

    pub fn flush(&self) -> anyhow::Result<()> {
        self.mmap.flush()?;
        self.env.force_sync()?;
        Ok(())
    }

    fn header(&self) -> &Header {
        unsafe { &*(self.mmap.as_ptr() as *const Header) }
    }

    fn slot_offset(&self, idx: usize) -> usize {
        HEADER_SIZE + idx * self.slot_size
    }

    fn read_slot_post_id(&self, idx: usize, ordering: Ordering) -> u64 {
        let off = self.slot_offset(idx);
        let atom = unsafe { &*(self.mmap.as_ptr().add(off) as *const AtomicU64) };
        atom.load(ordering)
    }

    fn write_slot_post_id(&self, idx: usize, post_id: u64, ordering: Ordering) {
        let off = self.slot_offset(idx);
        let atom = unsafe { &*(self.mmap.as_ptr().add(off) as *const AtomicU64) };
        atom.store(post_id, ordering);
    }

    fn write_slot_embedding(&self, idx: usize, embedding: &[f16]) {
        let off = self.slot_offset(idx) + SLOT_HEADER_SIZE;
        let bytes = std::mem::size_of_val(embedding);
        unsafe {
            std::ptr::copy_nonoverlapping(
                embedding.as_ptr() as *const u8,
                self.mmap.as_ptr().add(off) as *mut u8,
                bytes,
            );
        }
    }

    fn read_slot_embedding(&self, idx: usize, dst: &mut [f16]) {
        let off = self.slot_offset(idx) + SLOT_HEADER_SIZE;
        let src = unsafe {
            std::slice::from_raw_parts(self.mmap.as_ptr().add(off) as *const f16, self.emb_dim)
        };
        dst[..self.emb_dim].copy_from_slice(src);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tempfile::tempdir;

    const TEST_TTL: Duration = Duration::from_secs(2 * 24 * 3600);

    fn fresh_id(seq: u64) -> u64 {
        const TWEPOCH_MS: u64 = 1288834974657;
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        ((now_ms - TWEPOCH_MS) << 22) | (seq & 0x3FFFFF)
    }

    fn emb(seed: u64, dim: usize) -> Vec<f16> {
        (0..dim)
            .map(|i| f16::from_f32((seed.wrapping_add(i as u64) as f32) * 0.001))
            .collect()
    }

    #[test]
    fn basic_insert_get() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let c = LmdbEmbeddingCache::create(&lp, &dp, 1000, 4, TEST_TTL).unwrap();

        let id1 = fresh_id(1);
        let id2 = fresh_id(2);
        c.insert_many(vec![(id1, emb(10, 4)), (id2, emb(20, 4))]);
        assert_eq!(c.len(), 2);

        let mut dst = vec![f16::ZERO; 4];
        assert!(c.copy_to(id1, &mut dst));
        assert_eq!(dst, emb(10, 4));
        assert!(c.copy_to(id2, &mut dst));
        assert_eq!(dst, emb(20, 4));
        assert!(!c.copy_to(fresh_id(99), &mut dst));
    }

    #[test]
    fn update_existing() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let c = LmdbEmbeddingCache::create(&lp, &dp, 1000, 4, TEST_TTL).unwrap();

        let id1 = fresh_id(1);
        c.insert_many(vec![(id1, emb(10, 4))]);
        c.insert_many(vec![(id1, emb(999, 4))]);
        assert_eq!(c.len(), 1);

        let mut dst = vec![f16::ZERO; 4];
        assert!(c.copy_to(id1, &mut dst));
        assert_eq!(dst, emb(999, 4));
    }

    #[test]
    fn fifo_eviction() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let c = LmdbEmbeddingCache::create(&lp, &dp, 3, 4, TEST_TTL).unwrap();

        let ids: Vec<u64> = (1..=4).map(fresh_id).collect();
        c.insert_many(vec![
            (ids[0], emb(10, 4)),
            (ids[1], emb(20, 4)),
            (ids[2], emb(30, 4)),
        ]);
        assert_eq!(c.len(), 3);

        c.insert_many(vec![(ids[3], emb(40, 4))]);
        assert_eq!(c.len(), 3);

        let mut dst = vec![f16::ZERO; 4];
        assert!(!c.copy_to(ids[0], &mut dst));
        assert!(c.copy_to(ids[1], &mut dst));
        assert!(c.copy_to(ids[3], &mut dst));
        assert_eq!(dst, emb(40, 4));
    }

    #[test]
    #[ignore]
    fn persistent_across_open() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let id1 = fresh_id(1);
        {
            let c = LmdbEmbeddingCache::create(&lp, &dp, 1000, 4, TEST_TTL).unwrap();
            c.insert_many(vec![(id1, emb(10, 4))]);
            c.flush().unwrap();
        }
        let c = LmdbEmbeddingCache::open(&lp, &dp, TEST_TTL).unwrap();
        assert_eq!(c.len(), 1);
        let mut dst = vec![f16::ZERO; 4];
        assert!(c.copy_to(id1, &mut dst));
        assert_eq!(dst, emb(10, 4));
    }

    #[test]
    fn copy_batch_basic() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let c = LmdbEmbeddingCache::create(&lp, &dp, 1000, 4, TEST_TTL).unwrap();
        let id1 = fresh_id(1);
        let id2 = fresh_id(2);
        c.insert_many(vec![(id1, emb(10, 4)), (id2, emb(20, 4))]);

        let ids = vec![id1, fresh_id(99), id2];
        let mut out = vec![f16::ZERO; 3 * 4];
        let missing = c.copy_batch(&ids, 4, &mut out);
        assert_eq!(missing, 1);
        assert_eq!(&out[0..4], &emb(10, 4));
        assert_eq!(&out[8..12], &emb(20, 4));
    }

    #[test]
    fn skip_zero_post_id() {
        let dir = tempdir().unwrap();
        let lp = dir.path().join("idx.lmdb");
        let dp = dir.path().join("data.dat");
        let c = LmdbEmbeddingCache::create(&lp, &dp, 1000, 4, TEST_TTL).unwrap();
        c.insert_many(vec![(0, emb(10, 4)), (fresh_id(1), emb(20, 4))]);
        assert_eq!(c.len(), 1);
    }
}
