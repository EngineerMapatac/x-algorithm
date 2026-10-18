use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowlistEntry {
    pub added_by: String,
    pub reason: String,
    pub added_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowlistRecord {
    pub user_id: i64,
    #[serde(default)]
    pub added_by: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub added_at: u64,
    pub ttl_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowlistAddRequest {
    pub ttl_secs: u64,
    #[serde(default)]
    pub added_by: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowlistBulkRequest {
    pub entries: Vec<AllowlistBulkEntry>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllowlistBulkEntry {
    pub user_id: i64,
    pub ttl_secs: u64,
    #[serde(default)]
    pub added_by: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkResponse {
    pub dry_run: bool,
    pub total: usize,
    pub would_add: usize,
    pub would_update: usize,
    pub errors: usize,
    pub results: Vec<BulkRowResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BulkRowResult {
    pub user_id: i64,
    pub ok: bool,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub existing_ttl_secs: Option<i64>,
    #[serde(default)]
    pub ttl_secs: Option<u64>,
}

pub const MAX_TTL_SECS: u64 = 90 * 24 * 60 * 60;

pub fn parse_ttl(text: &str) -> Option<u64> {
    let s = text.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Ok(n) = s.parse::<u64>() {
        return Some(n);
    }
    let split = s.find(|c: char| !c.is_ascii_digit())?;
    let (num_part, unit_part) = s.split_at(split);
    let n: u64 = num_part.parse().ok()?;
    let mult: u64 = match unit_part.trim() {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86_400,
        "w" => 604_800,
        _ => return None,
    };
    n.checked_mul(mult)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BulkParseResult {
    pub rows: Vec<AllowlistBulkEntry>,
    pub errors: Vec<(usize, String)>,
}

pub fn parse_bulk_input(
    text: &str,
    default_ttl_secs: Option<u64>,
    added_by: &str,
) -> BulkParseResult {
    let mut out = BulkParseResult::default();

    for (idx, raw_line) in text.lines().enumerate() {
        let lineno = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<String> = if line.contains(',') {
            line.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            let mut iter = line.split_whitespace();
            let mut v = Vec::new();
            if let Some(t) = iter.next() {
                v.push(t.to_string());
            }
            if let Some(t) = iter.next() {
                v.push(t.to_string());
            }
            let rest = iter.collect::<Vec<_>>().join(" ");
            if !rest.is_empty() {
                v.push(rest);
            }
            v
        };

        let Some(user_id_str) = parts.first().map(|s| s.trim()) else {
            out.errors.push((lineno, "missing user_id".into()));
            continue;
        };
        let user_id: i64 = match user_id_str.parse() {
            Ok(n) => n,
            Err(_) => {
                out.errors.push((
                    lineno,
                    format!("user_id is not a valid integer: {user_id_str}"),
                ));
                continue;
            }
        };

        let ttl_text = parts
            .get(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let ttl_secs = if ttl_text.is_empty() {
            match default_ttl_secs {
                Some(n) => n,
                None => {
                    out.errors.push((
                        lineno,
                        format!("row has no TTL and no default TTL is set (user_id={user_id})"),
                    ));
                    continue;
                }
            }
        } else {
            match parse_ttl(&ttl_text) {
                Some(n) => n,
                None => {
                    out.errors.push((
                        lineno,
                        format!("invalid TTL `{ttl_text}` (use e.g. 30s, 5m, 2h, 7d, 2w)"),
                    ));
                    continue;
                }
            }
        };

        if ttl_secs == 0 {
            out.errors.push((lineno, "TTL must be > 0".into()));
            continue;
        }
        if ttl_secs > MAX_TTL_SECS {
            out.errors.push((
                lineno,
                format!("TTL exceeds 90-day max: {ttl_secs}s for user_id={user_id}"),
            ));
            continue;
        }

        let reason = parts.get(2).cloned().unwrap_or_default();

        out.rows.push(AllowlistBulkEntry {
            user_id,
            ttl_secs,
            added_by: added_by.to_string(),
            reason,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ttl_accepts_units() {
        assert_eq!(parse_ttl("30s"), Some(30));
        assert_eq!(parse_ttl("5m"), Some(300));
        assert_eq!(parse_ttl("2h"), Some(7200));
        assert_eq!(parse_ttl("7d"), Some(604_800));
        assert_eq!(parse_ttl("2w"), Some(1_209_600));
    }

    #[test]
    fn parse_ttl_accepts_plain_seconds() {
        assert_eq!(parse_ttl("3600"), Some(3600));
        assert_eq!(parse_ttl("0"), Some(0));
    }

    #[test]
    fn parse_ttl_is_case_insensitive_and_whitespace_tolerant() {
        assert_eq!(parse_ttl(" 7D "), Some(604_800));
        assert_eq!(parse_ttl("  24H"), Some(86_400));
    }

    #[test]
    fn parse_ttl_rejects_garbage() {
        assert_eq!(parse_ttl(""), None);
        assert_eq!(parse_ttl("abc"), None);
        assert_eq!(parse_ttl("7x"), None);
        assert_eq!(parse_ttl("d7"), None);
        assert_eq!(parse_ttl("-3d"), None);
    }

    fn rows(p: &BulkParseResult) -> Vec<(i64, u64, &str)> {
        p.rows
            .iter()
            .map(|r| (r.user_id, r.ttl_secs, r.reason.as_str()))
            .collect()
    }

    #[test]
    fn parse_bulk_input_csv_form() {
        let p = parse_bulk_input("12345, 7d, repeat offender\n67890, 24h, ", None, "alice");
        assert!(p.errors.is_empty(), "errors: {:?}", p.errors);
        assert_eq!(
            rows(&p),
            vec![(12345, 604_800, "repeat offender"), (67890, 86_400, ""),]
        );
        assert!(p.rows.iter().all(|r| r.added_by == "alice"));
    }

    #[test]
    fn parse_bulk_input_whitespace_form_keeps_multi_word_reason() {
        let p = parse_bulk_input("12345 7d some reason with spaces", None, "alice");
        assert!(p.errors.is_empty(), "errors: {:?}", p.errors);
        assert_eq!(rows(&p), vec![(12345, 604_800, "some reason with spaces")]);
    }

    #[test]
    fn parse_bulk_input_skips_blank_and_comment_lines() {
        let p = parse_bulk_input(
            "# header\n\n  \n12345, 7d\n# next group\n67890, 24h\n",
            None,
            "alice",
        );
        assert_eq!(rows(&p), vec![(12345, 604_800, ""), (67890, 86_400, "")]);
        assert!(p.errors.is_empty());
    }

    #[test]
    fn parse_bulk_input_default_ttl_fills_when_missing() {
        let p = parse_bulk_input("12345\n67890, 24h", Some(3600), "alice");
        assert!(p.errors.is_empty(), "errors: {:?}", p.errors);
        assert_eq!(rows(&p), vec![(12345, 3600, ""), (67890, 86_400, "")]);
    }

    #[test]
    fn parse_bulk_input_collects_all_errors_not_just_first() {
        let p = parse_bulk_input(
            "abc, 7d, bad id\n12345\n67890, junk, bad ttl",
            None,
            "alice",
        );
        assert_eq!(p.rows.len(), 0);
        assert_eq!(p.errors.len(), 3);
        assert_eq!(p.errors[0].0, 1);
        assert_eq!(p.errors[1].0, 2);
        assert_eq!(p.errors[2].0, 3);
    }

    #[test]
    fn parse_bulk_input_enforces_90d_cap() {
        let p = parse_bulk_input("12345, 100d", None, "alice");
        assert_eq!(p.rows.len(), 0);
        assert_eq!(p.errors.len(), 1);
        assert!(p.errors[0].1.contains("90-day max"));
    }

    #[test]
    fn parse_bulk_input_rejects_zero_ttl() {
        let p = parse_bulk_input("12345, 0", None, "alice");
        assert_eq!(p.rows.len(), 0);
        assert_eq!(p.errors.len(), 1);
        assert!(p.errors[0].1.contains("> 0"));
    }
}
