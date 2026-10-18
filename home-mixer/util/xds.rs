use crate::models::query::ScoredPostsQuery;
use xai_candidate_pipeline::component_library::route_gate::RouteGate;

pub const XDS_GATE: RouteGate = RouteGate::decider_only("enable_phoenix_xds_traffic");

pub const RETRIEVAL_XDS_GATE: RouteGate =
    RouteGate::decider_only("enable_phoenix_xds_retrieval_traffic");

pub const VM_RANKER_XDS_GATE: RouteGate = RouteGate::decider_only("enable_vm_ranker_xds_traffic");

pub const ENABLE_PHOENIX_XDS_TRAFFIC_DECIDER: &str = "enable_phoenix_xds_traffic";

pub fn use_xds_for_cluster(query: &ScoredPostsQuery, cluster_name: &str) -> bool {
    XDS_GATE.is_enabled(query, cluster_name)
}

pub fn use_xds_for_vm_ranker_cluster(query: &ScoredPostsQuery, cluster_name: &str) -> bool {
    VM_RANKER_XDS_GATE.is_enabled(query, cluster_name)
}
