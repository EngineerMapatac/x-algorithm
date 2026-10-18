use crate::{snowflake_ts_ms, UserArrowFeatures};
use std::collections::{HashMap, HashSet};

pub fn compute_derived(
    f: &UserArrowFeatures,
    n: usize,
    off: usize,
    rs: usize,
    burst: &mut [i32],
    streak: &mut [i32],
    transition: &mut [i32],
    eng_no_imp: &mut [bool],
    tweet_age: &mut [f32],
    dwell_ratio: &mut [f32],
    n_at: usize,
    dwell_norm: &[f32],
    serve_to_action: &[f32],
) {
    let server_actions: HashSet<i32> = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 61, 62, 70].into();
    let render_actions: HashSet<i32> = [11i32, 12].into();
    let mut rendered: HashSet<i64> = HashSet::new();

    for i in 0..n {
        let p = rs + off + i;

        let mut b = 0i32;
        let lo = i.saturating_sub(5);
        let hi = (i + 6).min(n);
        for j in lo..hi {
            if j == i {
                continue;
            }
            let ti = f.engagement_time_ms[i];
            let tj = f.engagement_time_ms[j];
            if ti > 0 && tj > 0 && (ti - tj).unsigned_abs() < 1000 {
                b += 1;
            }
        }
        burst[p] = b;

        let mut s = 1i32;
        if i > 0 && f.author_id[i] > 0 && f.author_id[i] == f.author_id[i - 1] {
            let mut j = i;
            while j > 0 && f.author_id[j] == f.author_id[j - 1] {
                s += 1;
                j -= 1;
            }
        }
        streak[p] = s;

        if i > 0 {
            transition[p] = (f.action_name[i - 1] * n_at as i32 + f.action_name[i]) % 4096;
        }

        let act = f.action_name[i];
        let tid = f.tweet_id[i];
        if server_actions.contains(&act) && tid > 0 && !rendered.contains(&tid) {
            eng_no_imp[p] = true;
        }
        if render_actions.contains(&act) && tid > 0 {
            rendered.insert(tid);
        }

        if tid > 0 && f.engagement_time_ms[i] > 0 {
            let created = snowflake_ts_ms(tid as u64);
            let eng = f.engagement_time_ms[i] as u64;
            if eng > created && created > 0 {
                let dm = (eng - created) as f64 / 60_000.0;
                tweet_age[p] = (dm.ln_1p() / 10.0).min(1.0) as f32;
            }
        }

        let d = dwell_norm[p];
        let sta = serve_to_action[p];
        if sta > 0.01 {
            dwell_ratio[p] = d / sta;
        }
    }
}

pub fn sequence_stats(f: &UserArrowFeatures, n: usize) -> (f32, f32, f32, f32, f32, f32) {
    if n < 2 {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let mut counts: HashMap<i32, u32> = HashMap::new();
    for i in 0..n {
        *counts.entry(f.action_name[i]).or_insert(0) += 1;
    }
    let total = n as f32;
    let entropy: f32 = counts
        .values()
        .map(|&c| {
            let p = c as f32 / total;
            if p > 0.0 {
                -p * p.ln()
            } else {
                0.0
            }
        })
        .sum();

    let mut deltas: Vec<f64> = Vec::new();
    for i in 1..n {
        let tc = f.engagement_time_ms[i];
        let tp = f.engagement_time_ms[i - 1];
        if tc > 0 && tp > 0 {
            deltas.push((tc - tp).max(0) as f64);
        }
    }
    let reg = if deltas.len() >= 2 {
        let mean = deltas.iter().sum::<f64>() / deltas.len() as f64;
        let var = deltas.iter().map(|&d| (d - mean).powi(2)).sum::<f64>() / deltas.len() as f64;
        (1.0 / (1.0 + var.sqrt())) as f32
    } else {
        0.0
    };

    let dwell_frac = f.dwell_time.iter().filter(|&&d| d > 0).count() as f32 / total;

    let ips: HashSet<i32> = f.ip_address.iter().filter(|&&h| h > 0).copied().collect();
    let uniq_ips = ips.len() as f32 / 100.0;

    let times: Vec<i64> = f
        .engagement_time_ms
        .iter()
        .filter(|&&t| t > 0)
        .copied()
        .collect();
    let span = if times.len() >= 2 {
        let hours =
            (*times.iter().max().unwrap() - *times.iter().min().unwrap()) as f64 / 3_600_000.0;
        (hours / 168.0) as f32
    } else {
        0.0
    };

    let mut tc: HashMap<i64, u32> = HashMap::new();
    for i in 0..n {
        let t = f.tweet_id[i];
        if t > 0 {
            *tc.entry(t).or_insert(0) += 1;
        }
    }
    let max_tc = tc.values().max().copied().unwrap_or(0);
    let rep = max_tc as f32 / total;

    (entropy, reg, dwell_frac, uniq_ips, span, rep)
}
