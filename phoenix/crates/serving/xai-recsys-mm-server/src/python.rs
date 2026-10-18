// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use half::f16;
use numpy::{PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1};
use pyo3::PyResult;
use pyo3::prelude::*;

use anyhow::{Context, Result};

use crate::mm_embedding_client::{MmEmbeddingsClient, get_mm_client};

#[pyclass(module = "xai_recsys_engine")]
pub struct PyMmEmbeddingsClient {
    client: MmEmbeddingsClient,
    runtime: tokio::runtime::Runtime,
    o2_preload_task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl PyMmEmbeddingsClient {
    pub fn client(&self) -> &MmEmbeddingsClient {
        &self.client
    }
}

#[pymethods]
impl PyMmEmbeddingsClient {
    #[new]
    #[pyo3(signature = (*, shared=false, worker_id=0, emb_dim=1024))]
    fn new(py: Python<'_>, shared: bool, worker_id: usize, emb_dim: usize) -> PyResult<Self> {
        xai_recsys_server::init_env_logger();

        let (runtime, client, o2_preload_task) = py
            .detach(|| -> Result<_> {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .worker_threads(4)
                    .thread_name("mm-thread")
                    .build()
                    .context("Failed to create tokio runtime")?;
                let (client, o2_preload_future) = get_mm_client(worker_id, emb_dim, shared)
                    .with_context(|| {
                        format!(
                            "Failed to create MM embeddings client: worker_id={}, emb_dim={}",
                            worker_id, emb_dim
                        )
                    })?;
                let o2_preload_task = runtime.spawn(o2_preload_future);
                Ok((runtime, client, o2_preload_task))
            })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        log::info!(
            "Created PyMmEmbeddingsClient (shared={}, worker_id={})",
            shared,
            worker_id,
        );

        Ok(Self {
            client,
            runtime,
            o2_preload_task,
        })
    }

    fn fetch_mm_embeddings_array<'py>(
        &self,
        py: Python<'py>,
        post_ids: PyReadonlyArray1<'py, i64>,
        emb_dim: usize,
    ) -> PyResult<Bound<'py, PyArray2<f16>>> {
        let ids: Vec<u64> = post_ids.as_slice()?.iter().map(|&x| x as u64).collect();
        let n = ids.len();
        let flat = py
            .detach(|| {
                self.runtime
                    .block_on(self.client.fetch_mm_embeddings(ids, emb_dim, n))
            })
            .map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "fetch_mm_embeddings failed: {}",
                    e
                ))
            })?;
        let arr = PyArray1::from_vec(py, flat);
        arr.reshape([n, emb_dim]).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!(
                "failed to reshape embeddings to ({n}, {emb_dim}): {e}"
            ))
        })
    }

    fn wait_until_ready(&mut self, py: Python<'_>) -> PyResult<()> {
        let result = py.detach(|| {
            let task = std::mem::replace(
                &mut self.o2_preload_task,
                self.runtime.spawn(async { Ok(()) }),
            );
            self.runtime.block_on(task)
        });
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "O2 preload failed: {}",
                e
            ))),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "O2 preload task join failed: {}",
                e
            ))),
        }
    }
}
