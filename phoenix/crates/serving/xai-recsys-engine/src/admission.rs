// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 X.AI Corp.
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdmissionEtaModel {
    #[default]
    Loop,
    Pipeline,
}

impl AdmissionEtaModel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Pipeline => "pipeline",
        }
    }
}

impl FromStr for AdmissionEtaModel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "loop" | "python" => Ok(Self::Loop),
            "pipeline" | "rust" => Ok(Self::Pipeline),
            other => Err(format!(
                "unknown admission eta model '{other}' (want loop|python or pipeline|rust)"
            )),
        }
    }
}

impl fmt::Display for AdmissionEtaModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AdmissionConfig {
    pub enabled: bool,
    pub batch_size: u64,
    pub margin_us: u64,
    pub post_us: u64,
    pub fallback_budget_us: u64,
    pub ewma_alpha: f64,
    pub pipeline_depth: u64,
    pub eta_model: AdmissionEtaModel,
}

pub struct AdmissionController {
    enabled: AtomicBool,
    batch_size: u64,
    margin_us: u64,
    post_us: u64,
    fallback_budget_us: u64,
    ewma_alpha: f64,
    pipeline_depth: u64,
    eta_model: AdmissionEtaModel,
    service_time_us: AtomicU64,
    pipeline_time_us: AtomicU64,
}

impl AdmissionController {
    pub fn new(cfg: AdmissionConfig) -> Self {
        let ewma_alpha = if cfg.ewma_alpha > 0.0 && cfg.ewma_alpha <= 1.0 {
            cfg.ewma_alpha
        } else {
            0.2
        };
        Self {
            enabled: AtomicBool::new(cfg.enabled),
            batch_size: cfg.batch_size.max(1),
            margin_us: cfg.margin_us,
            post_us: cfg.post_us,
            fallback_budget_us: cfg.fallback_budget_us,
            ewma_alpha,
            pipeline_depth: cfg.pipeline_depth,
            eta_model: cfg.eta_model,
            service_time_us: AtomicU64::new(0),
            pipeline_time_us: AtomicU64::new(0),
        }
    }

    pub fn eta_model(&self) -> AdmissionEtaModel {
        self.eta_model
    }

    fn ewma(prev: u64, sample: u64, alpha: f64) -> u64 {
        if prev == 0 {
            sample
        } else {
            (alpha * sample as f64 + (1.0 - alpha) * prev as f64) as u64
        }
    }

    pub fn reset_estimates(&self) {
        self.service_time_us.store(0, Ordering::Relaxed);
        self.pipeline_time_us.store(0, Ordering::Relaxed);
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn record_service_time_us(&self, sample_us: u64) {
        if sample_us == 0 {
            return;
        }
        let prev = self.service_time_us.load(Ordering::Relaxed);
        self.service_time_us.store(
            Self::ewma(prev, sample_us, self.ewma_alpha),
            Ordering::Relaxed,
        );
    }

    pub fn record_sojourn_us(&self, sample_us: u64, admit_backlog: i64) {
        if self.eta_model != AdmissionEtaModel::Pipeline || sample_us == 0 {
            return;
        }
        let q = admit_backlog.max(0) as u64;
        let residual = sample_us.saturating_sub(self.queue_wait_us(q));
        if residual == 0 {
            return;
        }
        let prev = self.pipeline_time_us.load(Ordering::Relaxed);
        self.pipeline_time_us.store(
            Self::ewma(prev, residual, self.ewma_alpha),
            Ordering::Relaxed,
        );
    }

    fn queue_wait_us(&self, q: u64) -> u64 {
        let s = self.service_time_us.load(Ordering::Relaxed);
        if s == 0 {
            0
        } else {
            q.saturating_mul(s) / self.batch_size
        }
    }

    pub fn should_reject(&self, remaining_us: Option<u64>, queue_depth: i64) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        let q = queue_depth.max(0) as u64;
        if q < self.batch_size {
            return false;
        }
        let remaining_us = match remaining_us {
            Some(r) => r,
            None => return false,
        };
        let eta_us = match self.predicted_eta_us(queue_depth) {
            Some(eta) => eta,
            None => return false,
        };
        eta_us.saturating_add(self.margin_us) > remaining_us
    }

    pub fn predicted_eta_us(&self, queue_depth: i64) -> Option<u64> {
        match self.eta_model {
            AdmissionEtaModel::Loop => self.predicted_eta_loop_us(queue_depth),
            AdmissionEtaModel::Pipeline => self.predicted_eta_pipeline_us(queue_depth),
        }
    }

    pub fn predicted_eta_loop_us(&self, queue_depth: i64) -> Option<u64> {
        let s = self.service_time_us.load(Ordering::Relaxed);
        if s == 0 {
            return None;
        }
        let q = queue_depth.max(0) as u64;
        let batches_ahead = (q + 1).div_ceil(self.batch_size);
        Some(
            batches_ahead
                .saturating_add(self.pipeline_depth)
                .saturating_mul(s)
                .saturating_add(self.post_us),
        )
    }

    pub fn predicted_eta_pipeline_us(&self, queue_depth: i64) -> Option<u64> {
        let p = self.pipeline_time_us.load(Ordering::Relaxed);
        if p == 0 {
            return None;
        }
        let q = queue_depth.max(0) as u64;
        Some(
            self.queue_wait_us(q)
                .saturating_add(p)
                .saturating_add(self.post_us),
        )
    }

    pub fn service_time_us(&self) -> u64 {
        self.service_time_us.load(Ordering::Relaxed)
    }

    pub fn pipeline_time_us(&self) -> u64 {
        self.pipeline_time_us.load(Ordering::Relaxed)
    }

    pub fn batch_size(&self) -> u64 {
        self.batch_size
    }

    pub fn margin_us(&self) -> u64 {
        self.margin_us
    }

    pub fn fallback_budget_us(&self) -> Option<u64> {
        if self.fallback_budget_us > 0 {
            Some(self.fallback_budget_us)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(enabled: bool) -> AdmissionConfig {
        AdmissionConfig {
            enabled,
            batch_size: 32,
            margin_us: 0,
            post_us: 0,
            fallback_budget_us: 0,
            ewma_alpha: 0.2,
            pipeline_depth: 0,
            eta_model: AdmissionEtaModel::Loop,
        }
    }

    fn cfg_pipeline(enabled: bool) -> AdmissionConfig {
        let mut c = cfg(enabled);
        c.eta_model = AdmissionEtaModel::Pipeline;
        c
    }

    #[test]
    fn disabled_never_rejects() {
        let c = AdmissionController::new(cfg(false));
        c.record_service_time_us(100_000);
        assert!(!c.should_reject(Some(1), 1_000_000));
    }

    #[test]
    fn no_estimate_admits() {
        let c = AdmissionController::new(cfg(true));
        assert!(!c.should_reject(Some(1), 1_000_000));
    }

    #[test]
    fn missing_deadline_without_fallback_admits() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        assert!(!c.should_reject(None, 1_000_000));
    }

    #[test]
    fn deep_backlog_tight_deadline_rejects() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        assert!(c.should_reject(Some(500_000), 320));
    }

    #[test]
    fn light_load_admits() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        assert!(!c.should_reject(Some(500_000), 0));
    }

    #[test]
    fn shallow_backlog_below_floor_admits() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        assert!(!c.should_reject(Some(1), 10));
    }

    #[test]
    fn fallback_budget_exposed_and_used() {
        let mut config = cfg(true);
        config.fallback_budget_us = 300_000;
        let c = AdmissionController::new(config);
        assert_eq!(c.fallback_budget_us(), Some(300_000));
        c.record_service_time_us(100_000);
        assert!(c.should_reject(Some(300_000), 320));
    }

    #[test]
    fn fallback_budget_none_when_disabled() {
        let c = AdmissionController::new(cfg(true));
        assert_eq!(c.fallback_budget_us(), None);
    }

    #[test]
    fn pipeline_depth_adds_completion_lag() {
        let mut config = cfg(true);
        config.pipeline_depth = 1;
        let c = AdmissionController::new(config);
        c.record_service_time_us(100_000);
        assert_eq!(c.predicted_eta_us(0), Some(200_000));
        assert_eq!(c.predicted_eta_us(320), Some(1_200_000));
        let cfg2 = {
            let mut x = cfg(true);
            x.pipeline_depth = 1;
            x
        };
        let bad = AdmissionController::new(cfg2);
        bad.record_service_time_us(500_000);
        assert_eq!(bad.predicted_eta_us(32), Some(1_500_000));
        assert!(bad.should_reject(Some(1_200_000), 32));
    }

    #[test]
    fn predicted_eta_tracks_backlog() {
        let c = AdmissionController::new(cfg(true));
        assert_eq!(c.predicted_eta_us(100), None);
        c.record_service_time_us(100_000);
        assert_eq!(c.service_time_us(), 100_000);
        assert_eq!(c.predicted_eta_us(0), Some(100_000));
        assert_eq!(c.predicted_eta_us(320), Some(1_100_000));
        assert_eq!(c.predicted_eta_us(-5), Some(100_000));
    }

    #[test]
    fn sojourn_replaces_pipeline_depth_term() {
        let c = AdmissionController::new(cfg_pipeline(true));
        c.record_service_time_us(23_000);
        c.record_sojourn_us(80_000, 9);
        let wait_9 = 9 * 23_000 / 32;
        let p = 80_000 - wait_9;
        assert_eq!(c.pipeline_time_us(), p);
        assert_eq!(c.predicted_eta_us(0), Some(p));
        assert_eq!(c.predicted_eta_us(9), Some(80_000));
        assert_eq!(c.predicted_eta_us(32), Some(23_000 + p));
    }

    #[test]
    fn sojourn_deep_queue_stores_residual() {
        let c = AdmissionController::new(cfg_pipeline(true));
        c.record_service_time_us(100_000);
        c.record_sojourn_us(500_000, 32);
        assert_eq!(c.pipeline_time_us(), 400_000);
        assert_eq!(c.predicted_eta_us(0), Some(400_000));
        assert_eq!(c.predicted_eta_us(32), Some(500_000));
    }

    #[test]
    fn sojourn_without_s_is_just_p() {
        let c = AdmissionController::new(cfg_pipeline(true));
        c.record_sojourn_us(75_000, 0);
        assert_eq!(c.predicted_eta_us(40), Some(75_000));
    }

    #[test]
    fn loop_model_ignores_sojourn_samples() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        c.record_sojourn_us(80_000, 0);
        assert_eq!(c.pipeline_time_us(), 0);
        assert_eq!(c.predicted_eta_us(0), Some(100_000));
        assert_eq!(c.predicted_eta_loop_us(0), Some(100_000));
        assert_eq!(c.predicted_eta_pipeline_us(0), None);
    }

    #[test]
    fn reset_estimates_clears_s_and_p() {
        let c = AdmissionController::new(cfg_pipeline(true));
        c.record_service_time_us(100_000);
        c.record_sojourn_us(80_000, 0);
        assert!(c.service_time_us() > 0);
        assert!(c.pipeline_time_us() > 0);
        assert!(c.should_reject(Some(1), 32));
        c.reset_estimates();
        assert_eq!(c.service_time_us(), 0);
        assert_eq!(c.pipeline_time_us(), 0);
        assert!(!c.should_reject(Some(1), 32));
    }

    #[test]
    fn eta_model_parses_aliases() {
        assert_eq!(
            "python".parse::<AdmissionEtaModel>().unwrap(),
            AdmissionEtaModel::Loop
        );
        assert_eq!(
            "rust".parse::<AdmissionEtaModel>().unwrap(),
            AdmissionEtaModel::Pipeline
        );
        assert!("nope".parse::<AdmissionEtaModel>().is_err());
    }

    #[test]
    fn ewma_is_idle_independent_and_smooths() {
        let c = AdmissionController::new(cfg(true));
        c.record_service_time_us(100_000);
        c.record_service_time_us(800_000);
        assert!(!c.should_reject(Some(500_000), 32));
    }
}
