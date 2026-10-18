# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import os

from xrex import settings


def get_cluster() -> str:
    return (
        os.environ.get("XAI_CLUSTER")
        or os.environ.get("K8S_CLUSTER_NAME")
        or os.environ.get("KUBE_CLUSTER_NAME")
        or ""
    ).strip()


def get_cloud() -> str:
    return (os.environ.get("XAI_CLOUD") or "").strip()


def get_user() -> str:
    return (os.environ.get("XAI_USER") or "").strip()


def get_job_name() -> str:
    return (os.environ.get("XAI_JOB_NAME") or "").strip()


def get_pod_name() -> str:
    return (os.environ.get("POD_NAME") or os.environ.get("KUBE_POD_NAME") or get_hostname()).strip()


def get_node_name() -> str:
    return (os.environ.get("KUBE_NODE_NAME") or "").strip()


def get_hostname() -> str:
    return (os.environ.get("HOSTNAME") or "").strip()


def has_roce_fabric() -> bool:
    return bool(settings.ROCE_FABRIC_CLOUD) and get_cloud() == settings.ROCE_FABRIC_CLOUD
