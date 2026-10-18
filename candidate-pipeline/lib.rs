#[cfg(feature = "quiet-spans")]
pub const SPAN_LEVEL: tracing::Level = tracing::Level::DEBUG;
#[cfg(not(feature = "quiet-spans"))]
pub const SPAN_LEVEL: tracing::Level = tracing::Level::INFO;

pub mod candidate_pipeline;
pub mod filter;
pub mod hydrator;
pub mod pipeline_summary;
pub mod query_hydrator;
pub mod scorer;
pub mod selector;
pub mod side_effect;
pub mod source;
pub mod util;
