// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use numpy::{PyArray1, PyReadonlyArray1, PyReadonlyArray2, ToPyArray};
use pyo3::PyResult;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use simd_adler32::adler32;

const PARALLEL_ADLER32_CHUNKS: usize = 128;

pub fn adler32_combine(checksum1: &mut u32, checksum2: u32, len2: usize) {
    let a1 = *checksum1 & 0xffff;
    let b1 = *checksum1 >> 16;
    let a2 = checksum2 & 0xffff;
    let b2 = checksum2 >> 16;
    let a = (a1 + a2 + 0xfff0) % 0xfff1;
    let b = (b1 + b2 + (a1 + 0xfff0) % 0xfff1 * (len2 % 0xfff1) as u32) % 0xfff1;
    *checksum1 = (b << 16) + a;
}

fn adler32_parallel_impl(slice: &[u8], num_chunks: usize) -> u32 {
    let checksums: Vec<_> = slice
        .par_chunks(slice.len().div_ceil(num_chunks))
        .map(|chunk| (adler32(&chunk), chunk.len()))
        .collect();
    checksums.into_iter().fold(1, |mut acc, (ck, len)| {
        adler32_combine(&mut acc, ck, len);
        acc
    })
}

#[pyfunction]
pub fn adler32_parallel<'py>(py: Python<'py>, arr: PyReadonlyArray1<'py, u8>) -> PyResult<u32> {
    let slice = arr.as_slice()?;
    let checksum = py.detach(|| adler32_parallel_impl(slice, PARALLEL_ADLER32_CHUNKS));
    Ok(checksum)
}

#[pyfunction]
pub fn adler32_combine_py(_py: Python<'_>, checksum1: u32, checksum2: u32, len2: usize) -> u32 {
    let mut result = checksum1;
    adler32_combine(&mut result, checksum2, len2);
    result
}

#[pyfunction]
pub fn adler32_shard_partial<'py>(
    py: Python<'py>,
    shard: PyReadonlyArray1<'py, u8>,
    shard_width: usize,
    shard_index: usize,
    total_cols: usize,
    num_chunks: usize,
) -> PyResult<Bound<'py, PyArray1<u32>>> {
    let slice = shard.as_slice()?;

    if shard_width == 0 {
        return Ok([0u32, 0u32].to_pyarray(py));
    }

    if slice.len() % shard_width != 0 {
        return Err(PyValueError::new_err(format!(
            "shard size {} must be divisible by shard_width {}",
            slice.len(),
            shard_width,
        )));
    }

    let num_rows = slice.len() / shard_width;
    let col_offset = shard_index * shard_width;

    let (byte_sum, pos_weighted_sum): (u32, u32) = py.detach(|| {
        let chunk_size = num_rows.div_ceil(num_chunks);
        slice
            .par_chunks(chunk_size * shard_width)
            .enumerate()
            .map(|(chunk_idx, chunk_data)| {
                let start_row = chunk_idx * chunk_size;
                let mut chunk_byte_sum = 0u64;
                let mut chunk_pos_sum = 0u64;

                for (local_row, row_data) in chunk_data.chunks_exact(shard_width).enumerate() {
                    let row = start_row + local_row;
                    let row_base = (row * total_cols + col_offset) as u64;

                    for (local_col, &byte) in row_data.iter().enumerate() {
                        let b = byte as u64;
                        chunk_byte_sum += b;
                        let pos = row_base + local_col as u64;
                        chunk_pos_sum = (chunk_pos_sum + pos * b) % 0xfff1;
                    }
                }

                ((chunk_byte_sum % 0xfff1) as u32, chunk_pos_sum as u32)
            })
            .reduce(
                || (0u32, 0u32),
                |(a1, b1), (a2, b2)| ((a1 + a2) % 0xfff1, (b1 + b2) % 0xfff1),
            )
    });

    Ok([byte_sum, pos_weighted_sum].to_pyarray(py))
}

#[pyfunction]
pub fn adler32_combine_partials<'py>(
    _py: Python<'py>,
    partials: PyReadonlyArray2<'py, u32>,
    total_len: usize,
) -> PyResult<u32> {
    let arr = partials.as_array();
    let num_shards = arr.shape()[0];

    if num_shards == 0 || total_len == 0 {
        return Ok(1);
    }

    let mut total_byte_sum = 0u32;
    let mut total_pos_sum = 0u32;

    for shard_idx in 0..num_shards {
        total_byte_sum = (total_byte_sum + arr[[shard_idx, 0]]) % 0xfff1;
        total_pos_sum = (total_pos_sum + arr[[shard_idx, 1]]) % 0xfff1;
    }

    let length = (total_len % 0xfff1) as u32;

    let a = (1 + total_byte_sum) % 0xfff1;

    let la = length * a % 0xfff1;
    let b = (la + 0xfff1 - total_pos_sum) % 0xfff1;

    Ok((b << 16) | a)
}
