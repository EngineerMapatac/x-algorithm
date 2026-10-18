# Quickstart

This walkthrough verifies the shipped nano ranking and retrieval paths with
synthetic data: train, checkpoint, resume, serve, then send a retrieve → rank
request. It is not a production-quality model or a production-scale setup.
Production data, checkpoints, orchestration, and scale are not included.

Run every command from this directory (the export root).

## Requirements

- Linux with an NVIDIA GPU and CUDA 12
- `uv` and Python 3.11 or newer
- A Rust toolchain and `protoc` 3.15 or newer

See [`README.md`](README.md) for system-package installation.

```bash
uv sync --extra engine
export PYTHONPATH=$PWD
```

The engine extra is required for training and serving. The first model run may
take a few minutes while JAX compiles.

Verify the install with random weights:

```bash
uv run python xrex/inference/oss_bench/bench.py --smoke --service_type ranking
```

## 1. Generate deterministic synthetic data

```bash
uv run python reference/world_snapshots.py --out ./synth_index --seed 20260721
export PHOENIX_INDEX_BASE=./synth_index

uv run python reference/dump_gen.py --out ./synth_dump --seed 20260721 \
  --num-rows 12288 --partitions 4 --rows-per-file 1024 \
  --sid ./synth_index/sid_snapshot/post_sid_v5_256x6.parquet --self-check
```

The same seed produces the same synthetic data. Keep
`PHOENIX_INDEX_BASE=./synth_index` set for the remaining commands.

## 2. Train the ranking model

```bash
uv run python reference/train_synth.py \
  --data ./synth_dump --steps 6 --out "$PWD/checkpoints" --metrics

uv run python -c 'import json; print(*((m["step"], m["loss"]) for m in map(json.loads, open("checkpoints/run/metrics.jsonl"))), sep="\n")'
```

The first command writes checkpoints under
`./checkpoints/home_direct_packed_nano_offline_kafka_dump/`. The second prints
the recorded step and loss values. The rehearsal observed loss decrease on this
synthetic run; do not treat six steps as evidence of model quality.

## 3. Resume training

Use the same output directory and increase the total step limit:

```bash
uv run python reference/train_synth.py \
  --data ./synth_dump --steps 12 --out "$PWD/checkpoints"
```

The trainer discovers the newest checkpoint. Its log includes:

```text
Discovered checkpoint to load:
CheckpointMeta(...)
Restored from checkpoint: <count> tensors ... checkpoint step is <N>
Checkpoint checksums match
create_dataset: resume_position={'last_batch_id': ..., ...}
```

Model state, optimizer state, and data position are restored. The synthetic dump
is finite; generate more rows before requesting a longer run.

## 4. Serve the ranking checkpoint

```bash
RANK_CKPT=$(ls -d "$PWD"/checkpoints/home_direct_packed_nano_offline_kafka_dump/elapsed_samples_*/*/ | sort | tail -1)

uv run python xrex/inference/oss_bench/bench.py \
  --checkpoint_path "$RANK_CKPT" \
  --service_type ranking \
  --config_name home_direct_packed_nano_offline_kafka_dump
```

`bench.py` restores the checkpoint, starts the gRPC server, sends one synthetic
request when the port is available, and then stops the server. The restore and
warmup log includes:

```text
Restored from checkpoint: <count> tensors ... checkpoint step is <N>
Checkpoint checksums match
gRPC server ready.
Model warm up finished.
```

Wait for port 9988 to be released before starting the full stack below.

## 5. Train retrieval and run retrieve → rank

Train the two-tower retrieval model on the same dump:

```bash
uv run python reference/train_synth.py \
  --config xrecsys_two_tower_nano_offline_kafka_dump \
  --data ./synth_dump --steps 6 --out "$PWD/checkpoints"
```

The retrieval checkpoint contains the candidate index built from the generated
synthetic snapshots.

Start the SID service and both model servers:

```bash
RANK_CKPT=$(ls -d "$PWD"/checkpoints/home_direct_packed_nano_offline_kafka_dump/elapsed_samples_*/*/ | sort | tail -1)
RETR_CKPT=$(ls -d "$PWD"/checkpoints/xrecsys_two_tower_nano_offline_kafka_dump/elapsed_samples_*/*/ | sort | tail -1)

uv run python reference/sid_index_server.py \
  --parquet ./synth_index/sid_snapshot/post_sid_v5_256x6.parquet --port 50061 &

XLA_PYTHON_CLIENT_MEM_FRACTION=0.30 uv run python xrex/inference/launch_inference.py \
  --driver local --service_type retrieval \
  --config_name xrecsys_two_tower_nano_offline_kafka_dump \
  --checkpoint_path "$RETR_CKPT" --grpc_port 9990 \
  --sid_endpoint localhost:50061 \
  --num_devices_per_process 1 --bs_per_device 1 \
  --history_seq_len 128 --candidate_seq_len 8 \
  --max_inflight_requests 16 --allow_random_init false --fake_mm_embeddings true \
  attn_impl=pallas_ranker_attn use_seqpack=False right_anchored_rope=True \
  bs_per_device=1 parallel_config.num_devices_per_process=1 num_devices_per_process=1 \
  ep=1 dp=1 training_ep=1 &

XLA_PYTHON_CLIENT_MEM_FRACTION=0.30 uv run python xrex/inference/launch_inference.py \
  --driver local --service_type ranking \
  --config_name home_direct_packed_nano_offline_kafka_dump \
  --checkpoint_path "$RANK_CKPT" --grpc_port 9988 --metrics_port 9091 \
  --num_devices_per_process 1 --bs_per_device 1 \
  --history_seq_len 128 --candidate_seq_len 16 \
  --max_inflight_requests 16 --allow_random_init false --fake_mm_embeddings true \
  attn_impl=pallas_ranker_attn_infer use_seqpack=False right_anchored_rope=True \
  bs_per_device=1 parallel_config.num_devices_per_process=1 num_devices_per_process=1 \
  ep=1 dp=1 training_ep=1 \
  model_config.model_config.sequence_len=146 &
```

Wait until both model servers log `Server ready to serve`, then send three
synthetic sessions through both real gRPC services:

```bash
uv run python reference/retrieve_then_rank.py \
  --data ./synth_dump --sessions 3 --topk 16 \
  --retrieval-port 9990 --ranking-port 9988
```

The client passes each session to retrieval, then asks ranking to score exactly
the returned candidates for the same user sequence. A successful run ends with:

```text
retrieve_then_rank: 3 session(s) completed the full loop.
```

This verifies the shipped integration only. The nano models and synthetic data
do not demonstrate recommendation quality, production performance, or scale.

## Next

- [`TRAINING.md`](TRAINING.md) explains the shipped training path.
- [`reference/README.md`](reference/README.md) describes the synthetic data and
  snapshot tools.
