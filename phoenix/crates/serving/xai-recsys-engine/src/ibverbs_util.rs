// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use ibverbs::{
    CompletionQueue, Context, PreparedQueuePair, ProtectionDomain, QueuePair, devices,
    ibv_access_flags, ibv_gid_type, ibv_mtu, ibv_qp_type, ibv_wc,
};
use lazy_static::lazy_static;
use std::{env, time::Duration};
use tokio::{task, time::timeout};
use tonic::Status;

pub const MAX_WR_SIZE: usize = 1 << 30;
pub const MAX_QUEUE_DEPTH: u32 = 1 << 10;

pub fn make_ib_contexts() -> Result<Vec<(Context, ProtectionDomain)>, Status> {
    if env::var("XAI_RECSYS_RDMA").as_deref() != Ok("1") {
        return Ok(Vec::new());
    }
    let devices = devices()?;
    let mut contexts = Vec::with_capacity(devices.len());
    for device in &devices {
        if device.name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.starts_with("mlx5_ib") || name.starts_with("mlx5_roce")
        }) {
            let ctx = device.open()?;
            let pd = ctx.alloc_pd()?;
            contexts.push((ctx, pd));
        }
    }
    Ok(contexts)
}

pub fn make_qp(
    is_send: bool,
    context: &Context,
    pd: &ProtectionDomain,
    max_inflight: u32,
    prio: usize,
) -> Result<(Vec<u8>, PreparedQueuePair, CompletionQueue), Status> {
    const MAX_SGE: u32 = 1;

    let Some(entry) = context.gid_table()?.into_iter().find(|entry| {
        entry.gid_type == ibv_gid_type::IBV_GID_TYPE_IB
            || (entry.gid_type == ibv_gid_type::IBV_GID_TYPE_ROCE_V2
                && entry.gid.subnet_prefix() == 0
                && entry.gid.interface_id() >> 32 == 0xFFFF)
    }) else {
        return Err(Status::internal("cannot find GID"));
    };
    let cq = context.create_cq(max_inflight as i32, 0)?;
    let mut qp_builder = pd.create_qp(&cq, &cq, ibv_qp_type::IBV_QPT_RC)?;
    let qp_builder = qp_builder
        .set_access(ibv_access_flags::IBV_ACCESS_LOCAL_WRITE)
        .allow_remote_rw()
        .set_gid_index(entry.gid_index)
        .set_min_rnr_timer(12)
        .set_max_rd_atomic(16)
        .set_max_dest_rd_atomic(16)
        .set_path_mtu(ibv_mtu::IBV_MTU_4096)
        .set_rnr_retry(7)
        .set_retry_count(7)
        .set_service_level((prio >> 8) as u8)
        .set_traffic_class((prio & 0xFF) as u8);
    let qp = if is_send {
        qp_builder
            .set_max_send_sge(MAX_SGE)
            .set_max_send_wr(MAX_QUEUE_DEPTH)
    } else {
        qp_builder
            .set_max_recv_sge(MAX_SGE)
            .set_max_recv_wr(MAX_QUEUE_DEPTH)
    }
    .build()?;
    let ep = qp.endpoint()?;
    let ep = bincode::serialize(&ep)
        .map_err(|e| Status::internal(format!("cannot serialize endpoint: {e}")))?;
    Ok((ep, qp, cq))
}

pub fn handshake(
    endpoints: Vec<Vec<u8>>,
    queues: Vec<(PreparedQueuePair, CompletionQueue)>,
) -> Result<Vec<(QueuePair, CompletionQueue)>, Status> {
    if endpoints.len() != queues.len() {
        return Err(Status::internal(format!(
            "{} endpoints but {} queue pairs",
            endpoints.len(),
            queues.len()
        )));
    }
    let mut out_queues = Vec::with_capacity(endpoints.len());
    for (ep, (qp, cq)) in endpoints.iter().zip(queues) {
        let ep = bincode::deserialize(ep)
            .map_err(|e| Status::internal(format!("cannot deserialize endpoint: {e}")))?;
        out_queues.push((qp.handshake(ep)?, cq));
    }
    Ok(out_queues)
}

lazy_static! {
    static ref RDMA_POLL_TIMEOUT: Duration = {
        std::env::var("COPY_PORT_RDMA_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(300))
    };
}

pub async fn poll(pending: usize, queues: &[(QueuePair, CompletionQueue)]) -> Result<(), Status> {
    match timeout(*RDMA_POLL_TIMEOUT, poll_inner(pending, queues)).await {
        Ok(result) => result,
        Err(_) => Err(Status::deadline_exceeded(format!(
            "copy_port RDMA poll timed out after {:?} with {} completions pending",
            *RDMA_POLL_TIMEOUT, pending
        ))),
    }
}

async fn poll_inner(
    mut pending: usize,
    queues: &[(QueuePair, CompletionQueue)],
) -> Result<(), Status> {
    while pending != 0 {
        let mut count = 0;
        for (_, cq) in queues {
            let mut completions = [ibv_wc::default(); MAX_QUEUE_DEPTH as usize];
            let completed = cq.poll(&mut completions[..])?;
            count += completed.len();
            for wr in completed {
                if let Some(err) = wr.error() {
                    return Err(Status::internal(format!("completion error: {err:?}")));
                }
            }
        }
        pending -= count;
        if count == 0 {
            task::yield_now().await;
        }
    }
    Ok(())
}
