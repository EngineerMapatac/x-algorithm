mod derived;

use std::collections::HashSet;
use std::io::Cursor;

use arrow::array::*;
use arrow::ipc::reader::FileReader;
use numpy::{PyArray1, PyArray2, PyArray3};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use sha2::{Digest, Sha256};

fn get_i32(s: &StructArray, name: &str) -> Vec<i32> {
    match s.column_by_name(name) {
        Some(col) => {
            let n = col.len();
            if let Some(a) = col.as_any().downcast_ref::<UInt16Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i32 } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<UInt32Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i32 } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<Int32Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<UInt8Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i32 } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<Int16Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i32 } else { 0 })
                    .collect();
            }
            if let Some(dict) = col
                .as_any()
                .downcast_ref::<DictionaryArray<arrow::datatypes::UInt16Type>>()
            {
                if let Some(values) = dict.values().as_any().downcast_ref::<UInt64Array>() {
                    return (0..n)
                        .map(|i| {
                            if dict.is_valid(i) {
                                values.value(dict.keys().value(i) as usize) as i32
                            } else {
                                0
                            }
                        })
                        .collect();
                }
            }
            vec![0i32; n]
        }
        None => vec![0i32; s.len()],
    }
}

fn get_i64(s: &StructArray, name: &str) -> Vec<i64> {
    match s.column_by_name(name) {
        Some(col) => {
            let n = col.len();
            if let Some(a) = col.as_any().downcast_ref::<UInt64Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i64 } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<Int64Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) } else { 0 })
                    .collect();
            }
            if let Some(a) = col.as_any().downcast_ref::<UInt32Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) as i64 } else { 0 })
                    .collect();
            }
            vec![0i64; n]
        }
        None => vec![0i64; s.len()],
    }
}

fn get_f32(s: &StructArray, name: &str) -> Vec<f32> {
    match s.column_by_name(name) {
        Some(col) => {
            let n = col.len();
            if let Some(a) = col.as_any().downcast_ref::<Float32Array>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) } else { 0.0 })
                    .collect();
            }
            vec![0f32; n]
        }
        None => vec![0f32; s.len()],
    }
}

fn get_bool(s: &StructArray, name: &str) -> Vec<bool> {
    match s.column_by_name(name) {
        Some(col) => {
            let n = col.len();
            if let Some(a) = col.as_any().downcast_ref::<BooleanArray>() {
                return (0..n)
                    .map(|i| if a.is_valid(i) { a.value(i) } else { false })
                    .collect();
            }
            vec![false; n]
        }
        None => vec![false; s.len()],
    }
}

fn get_string(s: &StructArray, name: &str) -> Vec<String> {
    match s.column_by_name(name) {
        Some(col) => {
            let n = col.len();
            if let Some(a) = col.as_any().downcast_ref::<StringArray>() {
                return (0..n)
                    .map(|i| {
                        if a.is_valid(i) {
                            a.value(i).to_string()
                        } else {
                            String::new()
                        }
                    })
                    .collect();
            }
            vec![String::new(); n]
        }
        None => vec![String::new(); s.len()],
    }
}

fn hash_str(s: &str, modulus: u32) -> i32 {
    if s.is_empty() {
        return 0;
    }
    let h = u32::from_be_bytes(Sha256::digest(s.as_bytes())[..4].try_into().unwrap());
    (h % modulus) as i32
}

fn hash_i64(val: i64, modulus: u32) -> i32 {
    if val == 0 {
        return 0;
    }
    let h = u32::from_be_bytes(Sha256::digest(val.to_be_bytes())[..4].try_into().unwrap());
    (h % modulus) as i32
}

fn snowflake_ts_ms(tweet_id: u64) -> u64 {
    if tweet_id == 0 {
        return 0;
    }
    (tweet_id >> 22) + 1288834974657
}

fn action_name_to_group(action_name: i32) -> i32 {
    match action_name {
        1 | 2 => 1,
        4 | 7 => 2,
        6 | 9 => 3,
        29 | 30 | 14 => 4,
        11 | 12 => 5,
        31 | 37 => 6,
        36 | 70 => 7,
        21 | 61 | 62 => 8,
        10 | 16 => 9,
        23 | 63 => 10,
        25 => 11,
        5 | 8 => 3,
        _ => 0,
    }
}

fn action_name_to_category(action_name: i32) -> i32 {
    match action_name {
        61 | 62 | 63 => 2,
        _ => 1,
    }
}

fn client_app_id_to_platform(app_id: i32) -> i32 {
    match app_id {
        258901 => 1,
        3033300 => 2,
        _ if app_id > 0 && app_id < 1000 => 4,
        _ => 0,
    }
}

struct UserArrowFeatures {
    n: usize,
    action_name: Vec<i32>,
    tweet_id: Vec<i64>,
    author_id: Vec<i64>,
    engagement_time_ms: Vec<i64>,
    dwell_time: Vec<i32>,
    product_surface: Vec<i32>,
    client_app_id: Vec<i32>,
    is_following: Vec<bool>,
    fav_count: Vec<i64>,
    rt_count: Vec<i64>,
    reply_count: Vec<i64>,
    quote_count: Vec<i64>,
    view_count: Vec<i64>,
    has_media: Vec<bool>,
    language_code: Vec<i32>,
    promoted_id: Vec<i64>,
    tweet_position: Vec<i32>,
    first_video_duration: Vec<i64>,
    quoting_author_id: Vec<i64>,
    quoted_author_id: Vec<i64>,
    in_reply_to_author_id: Vec<i64>,
    retweeting_author_id: Vec<i64>,
    page: Vec<i32>,
    ip_address: Vec<i32>,
    country_code: Vec<i32>,
    timezone: Vec<i32>,
    served_time_ms: Vec<i64>,
    search_query: Vec<String>,
    section: Vec<String>,
    external_link_duration: Vec<i64>,
    device_battery: Vec<f32>,
    device_brightness: Vec<f32>,
    device_charging: Vec<bool>,
    device_network: Vec<i32>,
    device_max_storage: Vec<f32>,
    device_available_storage: Vec<f32>,
}

fn extract_single_user_arrow(
    raw: &[u8],
    seq_len: usize,
    _n_at: usize,
) -> anyhow::Result<UserArrowFeatures> {
    let decompressed = zstd::decode_all(Cursor::new(raw))?;
    let cursor = Cursor::new(decompressed);
    let reader = FileReader::try_new(cursor, None)?;
    let mut batches = Vec::new();
    for b in reader {
        batches.push(b?);
    }
    if batches.is_empty() {
        anyhow::bail!("empty");
    }
    let batch = if batches.len() == 1 {
        batches.into_iter().next().unwrap()
    } else {
        arrow::compute::concat_batches(&batches[0].schema(), &batches)?
    };

    let actions_col = batch
        .column_by_name("user_actions")
        .ok_or_else(|| anyhow::anyhow!("no user_actions"))?;
    let list = actions_col
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| anyhow::anyhow!("not ListArray"))?;
    if list.is_empty() || list.is_null(0) {
        anyhow::bail!("empty list");
    }

    let vals = list.value(0);
    let sa = vals
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| anyhow::anyhow!("not StructArray"))?;
    let total = sa.len();
    if total == 0 {
        anyhow::bail!("zero actions");
    }

    let start = if total > seq_len { total - seq_len } else { 0 };
    let take = total.min(seq_len);
    let sl = sa.slice(start, take);
    let sl = sl.as_any().downcast_ref::<StructArray>().unwrap();
    let n = take;

    Ok(UserArrowFeatures {
        n,
        action_name: get_i32(sl, "action_name"),
        tweet_id: get_i64(sl, "tweet_id"),
        author_id: get_i64(sl, "author_id"),
        engagement_time_ms: get_i64(sl, "engagement_time_ms"),
        dwell_time: get_i32(sl, "dwell_time"),
        product_surface: get_i32(sl, "product_surface"),
        client_app_id: get_i32(sl, "client_app_id"),
        is_following: get_bool(sl, "is_author_followed_by_user"),
        fav_count: get_i64(sl, "favorite_count"),
        rt_count: get_i64(sl, "retweet_count"),
        reply_count: get_i64(sl, "reply_count"),
        quote_count: get_i64(sl, "quote_count"),
        view_count: get_i64(sl, "view_count"),
        has_media: get_bool(sl, "has_media"),
        language_code: get_i32(sl, "post_language_code"),
        promoted_id: get_i64(sl, "promoted_id"),
        tweet_position: get_i32(sl, "tweet_position"),
        first_video_duration: get_i64(sl, "first_video_duration_ms"),
        quoting_author_id: get_i64(sl, "quoting_author_id"),
        quoted_author_id: get_i64(sl, "quoted_author_id"),
        in_reply_to_author_id: get_i64(sl, "in_reply_to_author_id"),
        retweeting_author_id: get_i64(sl, "retweeting_author_id"),
        page: get_i32(sl, "page"),
        ip_address: get_i32(sl, "ip_address"),
        country_code: get_i32(sl, "engaging_ip_country_code"),
        timezone: get_i32(sl, "timezone"),
        served_time_ms: get_i64(sl, "served_time_ms"),
        search_query: get_string(sl, "search_query"),
        section: get_string(sl, "section"),
        external_link_duration: get_i64(sl, "external_link_session_duration"),
        device_battery: get_f32(sl, "device_battery_level"),
        device_brightness: get_f32(sl, "device_brightness_level"),
        device_charging: get_bool(sl, "device_is_charging"),
        device_network: get_i32(sl, "device_network_type"),
        device_max_storage: get_f32(sl, "device_max_storage"),
        device_available_storage: get_f32(sl, "device_available_storage"),
    })
}

#[pyfunction]
#[pyo3(signature = (user_rows, seq_len=4096, n_action_types=128, n_labels=20))]
fn extract_batch<'py>(
    py: Python<'py>,
    user_rows: Vec<(i64, Vec<u8>)>,
    seq_len: usize,
    n_action_types: usize,
    n_labels: usize,
) -> PyResult<Bound<'py, PyDict>> {
    let b = user_rows.len();
    let s = seq_len;
    let n_at = n_action_types;

    let mut user_ids = vec![0i64; b];
    let mut tweet_ids = vec![0i64; b * s];
    let mut author_ids = vec![0i64; b * s];
    let mut action_multihot = vec![false; b * s * n_at];
    let mut time_delta = vec![0f32; b * s];
    let mut dwell_norm = vec![0f32; b * s];
    let mut product_surface = vec![0i32; b * s];
    let mut client_app_id = vec![0i32; b * s];
    let mut hour_of_day = vec![0i32; b * s];
    let mut had_prior_render = vec![false; b * s];
    let mut impr_ts = vec![0i32; b * s];
    let mut padding_mask = vec![false; b * s];
    let mut dev_battery = vec![0f32; b * s];
    let mut dev_brightness = vec![0f32; b * s];
    let mut dev_charging = vec![false; b * s];
    let mut dev_network = vec![0i32; b * s];
    let mut dev_storage = vec![0f32; b * s];
    let mut is_following = vec![false; b * s];
    let mut popularity = vec![0f32; b * s];
    let mut has_media = vec![false; b * s];
    let mut post_lang = vec![0i32; b * s];
    let mut serve_to_action = vec![0f32; b * s];
    let mut ip_hash = vec![0i32; b * s];
    let mut page_ctx = vec![0i32; b * s];
    let mut search_hash = vec![0i32; b * s];
    let mut ext_link_dur = vec![0f32; b * s];
    let mut feed_pos = vec![0i32; b * s];
    let mut is_promoted = vec![false; b * s];
    let mut video_dur = vec![0f32; b * s];
    let mut section_hash = vec![0i32; b * s];
    let mut country = vec![0i32; b * s];
    let mut tz = vec![0i32; b * s];
    let mut view_ct = vec![0f32; b * s];
    let mut quoting_ah = vec![0i32; b * s];
    let mut in_reply_ah = vec![0i32; b * s];
    let mut retweeting_ah = vec![0i32; b * s];
    let mut quoted_ah = vec![0i32; b * s];
    let mut burst = vec![0i32; b * s];
    let mut streak = vec![0i32; b * s];
    let mut transition = vec![0i32; b * s];
    let mut eng_no_imp = vec![false; b * s];
    let mut tweet_age = vec![0f32; b * s];
    let mut dwell_ratio = vec![0f32; b * s];
    let mut action_grp = vec![0i32; b * s];
    let mut action_cat = vec![0i32; b * s];
    let mut client_plat = vec![0i32; b * s];
    let mut entropy = vec![0f32; b * s];
    let mut regularity = vec![0f32; b * s];
    let mut dwell_frac = vec![0f32; b * s];
    let mut uniq_ips = vec![0f32; b * s];
    let mut time_span = vec![0f32; b * s];
    let mut tweet_rep = vec![0f32; b * s];

    for (bi, (uid, raw)) in user_rows.iter().enumerate() {
        user_ids[bi] = *uid;
        let f = match extract_single_user_arrow(raw, s, n_at) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let n = f.n;
        let off = s - n;
        let rs = bi * s;

        for i in 0..n {
            let p = rs + off + i;
            padding_mask[p] = true;

            let act = f.action_name[i].clamp(0, n_at as i32 - 1) as usize;
            if act > 0 {
                action_multihot[bi * s * n_at + (off + i) * n_at + act] = true;
            }

            tweet_ids[p] = f.tweet_id[i];
            author_ids[p] = f.author_id[i];
            product_surface[p] = f.product_surface[i];
            client_app_id[p] = f.client_app_id[i];
            if f.engagement_time_ms[i] > 0 {
                impr_ts[p] = (f.engagement_time_ms[i] / 1000) as i32;
                hour_of_day[p] = ((f.engagement_time_ms[i] / 3_600_000) % 24) as i32;
            }

            dev_battery[p] = f.device_battery[i];
            dev_brightness[p] = f.device_brightness[i];
            dev_charging[p] = f.device_charging[i];
            dev_network[p] = f.device_network[i];
            if f.device_max_storage[i] > 0.0 {
                dev_storage[p] = f.device_available_storage[i] / f.device_max_storage[i];
            }

            is_following[p] = f.is_following[i];
            let total_eng = f.fav_count[i]
                + f.rt_count[i]
                + f.reply_count[i]
                + f.quote_count[i]
                + f.view_count[i];
            popularity[p] = ((total_eng as f64 + 1.0).log10() / 7.0) as f32;
            has_media[p] = f.has_media[i];
            post_lang[p] = f.language_code[i];

            if f.served_time_ms[i] > 0 && f.engagement_time_ms[i] > f.served_time_ms[i] {
                let d = (f.engagement_time_ms[i] - f.served_time_ms[i]) as f64;
                serve_to_action[p] = (d.ln_1p() / 300_000f64.ln_1p()) as f32;
            }
            ip_hash[p] = f.ip_address[i];

            page_ctx[p] = f.page[i];
            search_hash[p] = hash_str(&f.search_query[i], 1024);
            if f.external_link_duration[i] > 0 {
                ext_link_dur[p] =
                    ((f.external_link_duration[i] as f64).ln_1p() / 300_000f64.ln_1p()) as f32;
            }
            feed_pos[p] = f.tweet_position[i];
            is_promoted[p] = f.promoted_id[i] > 0;
            if f.first_video_duration[i] > 0 {
                video_dur[p] =
                    ((f.first_video_duration[i] as f64 / 1000.0).ln_1p() / 300.0f64.ln_1p()) as f32;
            }
            section_hash[p] = hash_str(&f.section[i], 256);

            country[p] = f.country_code[i];
            tz[p] = f.timezone[i];

            view_ct[p] = ((f.view_count[i] as f64 + 1.0).log10() / 7.0) as f32;
            quoting_ah[p] = hash_i64(f.quoting_author_id[i], 65536);
            in_reply_ah[p] = hash_i64(f.in_reply_to_author_id[i], 65536);
            retweeting_ah[p] = hash_i64(f.retweeting_author_id[i], 65536);
            quoted_ah[p] = hash_i64(f.quoted_author_id[i], 65536);

            action_grp[p] = action_name_to_group(f.action_name[i]);
            action_cat[p] = action_name_to_category(f.action_name[i]);
            client_plat[p] = client_app_id_to_platform(f.client_app_id[i]);
        }

        for i in 1..n {
            let p = rs + off + i;
            let tc = f.engagement_time_ms[i];
            let tp = f.engagement_time_ms[i - 1];
            if tc > 0 && tp > 0 {
                let dm = ((tc - tp).max(0) as f64) / 60_000.0;
                time_delta[p] = (dm.ln_1p().min(10.0) / 10.0) as f32;
            }
        }
        for i in 0..n {
            let p = rs + off + i;
            let d = f.dwell_time[i];
            if d > 0 {
                dwell_norm[p] = ((d as f32 / 1000.0).ln_1p()) / 300.0f32.ln_1p();
            }
        }

        let render_set: HashSet<i32> = [11i32, 12].into();
        let mut seen: HashSet<i64> = HashSet::new();
        for i in 0..n {
            let p = rs + off + i;
            let tid = f.tweet_id[i];
            if tid > 0 && seen.contains(&tid) {
                had_prior_render[p] = true;
            }
            if render_set.contains(&f.action_name[i]) && tid > 0 {
                seen.insert(tid);
            }
        }

        derived::compute_derived(
            &f,
            n,
            off,
            rs,
            &mut burst,
            &mut streak,
            &mut transition,
            &mut eng_no_imp,
            &mut tweet_age,
            &mut dwell_ratio,
            n_at,
            &dwell_norm,
            &serve_to_action,
        );

        let stats = derived::sequence_stats(&f, n);
        for i in 0..n {
            let p = rs + off + i;
            entropy[p] = stats.0;
            regularity[p] = stats.1;
            dwell_frac[p] = stats.2;
            uniq_ips[p] = stats.3;
            time_span[p] = stats.4;
            tweet_rep[p] = stats.5;
        }
    }

    let dict = PyDict::new(py);
    dict.set_item("user_ids_raw", PyArray1::from_vec(py, user_ids))?;

    macro_rules! a2d {
        ($d:expr, $n:expr, $v:expr) => {
            $d.set_item(
                $n,
                PyArray2::from_vec2(py, &$v.chunks(s).map(|c| c.to_vec()).collect::<Vec<_>>())
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
            )?;
        };
    }

    a2d!(dict, "tweet_ids_raw", tweet_ids);
    a2d!(dict, "author_ids_raw", author_ids);

    let aseq = PyDict::new(py);

    aseq.set_item(
        "action_types",
        PyArray3::from_vec3(
            py,
            &action_multihot
                .chunks(s * n_at)
                .map(|u| u.chunks(n_at).map(|p| p.to_vec()).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
    )?;

    let cont: Vec<Vec<f32>> = time_delta
        .iter()
        .zip(dwell_norm.iter())
        .map(|(&t, &d)| vec![t, d])
        .collect();
    aseq.set_item(
        "continuous_actions",
        PyArray3::from_vec3(py, &cont.chunks(s).map(|c| c.to_vec()).collect::<Vec<_>>())
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
    )?;

    a2d!(aseq, "product_surface", product_surface);
    a2d!(aseq, "client_app_id", client_app_id);
    a2d!(aseq, "hour_of_day", hour_of_day);
    a2d!(aseq, "had_prior_render", had_prior_render);
    a2d!(aseq, "impr_ts", impr_ts);
    a2d!(aseq, "padding_mask", padding_mask);
    a2d!(aseq, "device_battery", dev_battery);
    a2d!(aseq, "device_brightness", dev_brightness);
    a2d!(aseq, "device_charging", dev_charging);
    a2d!(aseq, "device_network", dev_network);
    a2d!(aseq, "device_storage_ratio", dev_storage);
    a2d!(aseq, "is_following_author", is_following);
    a2d!(aseq, "target_popularity", popularity);
    a2d!(aseq, "target_has_media", has_media);
    a2d!(aseq, "post_language", post_lang);
    a2d!(aseq, "serve_to_action_ms", serve_to_action);
    a2d!(aseq, "ip_hash", ip_hash);
    a2d!(aseq, "page_context", page_ctx);
    a2d!(aseq, "search_query_hash", search_hash);
    a2d!(aseq, "external_link_duration", ext_link_dur);
    a2d!(aseq, "tweet_feed_position", feed_pos);
    a2d!(aseq, "is_promoted_tweet", is_promoted);
    a2d!(aseq, "video_duration_ms", video_dur);
    a2d!(aseq, "section_hash", section_hash);
    a2d!(aseq, "country_code", country);
    a2d!(aseq, "timezone", tz);
    a2d!(aseq, "view_count", view_ct);
    a2d!(aseq, "quoting_author_hash", quoting_ah);
    a2d!(aseq, "in_reply_to_author_hash", in_reply_ah);
    a2d!(aseq, "retweeting_author_hash", retweeting_ah);
    a2d!(aseq, "quoted_author_hash", quoted_ah);
    a2d!(aseq, "action_burst_count", burst);
    a2d!(aseq, "same_author_streak", streak);
    a2d!(aseq, "action_transition", transition);
    a2d!(aseq, "engagement_without_impression", eng_no_imp);
    a2d!(aseq, "time_since_tweet_created", tweet_age);
    a2d!(aseq, "dwell_to_action_ratio", dwell_ratio);
    a2d!(aseq, "action_group", action_grp);
    a2d!(aseq, "action_category", action_cat);
    a2d!(aseq, "client_platform", client_plat);
    a2d!(aseq, "action_entropy", entropy);
    a2d!(aseq, "timing_regularity", regularity);
    a2d!(aseq, "dwell_fraction", dwell_frac);
    a2d!(aseq, "unique_ips", uniq_ips);
    a2d!(aseq, "sequence_time_span", time_span);
    a2d!(aseq, "target_tweet_repetition", tweet_rep);

    dict.set_item("action_seq", aseq)?;

    let uf = PyDict::new(py);
    uf.set_item("account_age_norm", PyArray1::from_vec(py, vec![0f32; b]))?;
    uf.set_item("followers_log", PyArray1::from_vec(py, vec![0f32; b]))?;
    uf.set_item("is_verified", PyArray1::from_vec(py, vec![0f32; b]))?;
    uf.set_item("user_cred_norm", PyArray1::from_vec(py, vec![0f32; b]))?;
    uf.set_item("app_reputation", PyArray1::from_vec(py, vec![0f32; b]))?;
    dict.set_item("user_features", uf)?;

    dict.set_item(
        "label_vector",
        PyArray2::from_vec2(py, &vec![vec![false; n_labels]; b])
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
    )?;
    dict.set_item(
        "label_mask",
        PyArray2::from_vec2(py, &vec![vec![false; n_labels]; b])
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
    )?;

    Ok(dict)
}

#[pymodule]
fn abuse_v3_features(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_batch, m)?)?;
    Ok(())
}
