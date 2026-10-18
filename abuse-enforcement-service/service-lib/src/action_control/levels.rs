use super::*;
use crate::confidence::{self, Claim, RowOutcome};

pub fn level_lkey(
    entity: ControlEntity,
    head: &str,
    classifier: Option<&str>,
    row: LevelRow,
) -> String {
    let e = entity.as_str();
    let r = row.as_str();
    match classifier {
        None => format!("{e}/l/{head}/{r}"),
        Some(c) => format!("{e}/cl/{head}/{c}/{r}"),
    }
}

impl Snapshot {
    fn first<'a>(&'a self, keys: impl Iterator<Item = String>) -> Option<Hit<'a>> {
        keys.into_iter().find_map(|k| self.at(&k))
    }

                    pub(super) fn mover(
        &self,
        entity: ControlEntity,
        names: &[&str],
        classifier: Option<&str>,
    ) -> Option<(&str, &ActionControlRecord)> {
        if let Some(c) = classifier {
            match self.first(names.iter().map(|n| lkey_for(entity, n, Some(c), None))) {
                Some(Hit::Record(k, r)) if r.mode == ControlMode::Rows => return Some((k, r)),
                Some(Hit::Record(_, r)) if r.mode == ControlMode::Rules => {}
                Some(_) => return None,
                None => {}
            }
        }
        match self.first(names.iter().map(|n| lkey_for(entity, n, None, None))) {
            Some(Hit::Record(k, r)) if r.mode == ControlMode::Rows => Some((k, r)),
            _ => None,
        }
    }

                    pub(super) fn row_decider(
        &self,
        entity: ControlEntity,
        names: &[&str],
        classifier: Option<&str>,
        row: LevelRow,
        recording: &ActionControlRecord,
    ) -> Option<ActionControlRecord> {
        if let Some((_, scope)) = self.mover(entity, names, classifier) {
            let lkey = level_lkey(entity, &scope.head, scope.classifier.as_deref(), row);
            return Some(match self.at(&lkey) {
                Some(Hit::Record(_, r)) if r.mode != ControlMode::Rows => r.clone(),
                _ => recording.clone(),
            });
        }
        match self.plain_decider(
            entity,
            names[0],
            names.get(1).copied(),
            classifier.unwrap_or(""),
            &[],
        ) {
            Some(Hit::Record(_, r)) if r.mode != ControlMode::Rows => Some(r.clone()),
            Some(_) => Some(recording.clone()),
            None => None,
        }
    }

                        pub(super) fn under_scope(
        &self,
        entity: ControlEntity,
        names: &[&str],
        classifier: Option<&str>,
    ) -> Vec<(String, Option<ControlMode>)> {
        let e = entity.as_str();
        let mut prefixes = Vec::new();
        for n in names {
            match classifier {
                Some(c) => {
                    prefixes.push(format!("{e}/b/{n}/"));
                    prefixes.push(format!("{e}/cb/{n}/{c}/"));
                }
                None => {
                    prefixes.push(format!("{e}/b/{n}/"));
                    prefixes.push(format!("{e}/cb/{n}/"));
                }
            }
        }
        let classifiers: Vec<String> = match classifier {
            None => names.iter().map(|n| format!("{e}/c/{n}/")).collect(),
            Some(_) => Vec::new(),
        };
        let mut out: Vec<(String, Option<ControlMode>)> = self
            .records
            .iter()
            .filter(|(k, r)| {
                prefixes.iter().any(|p| k.starts_with(p))
                    || (r.mode == ControlMode::Rules
                        && classifiers.iter().any(|p| k.starts_with(p)))
            })
            .map(|(k, r)| (k.clone(), Some(r.mode)))
            .chain(
                self.unreadable
                    .keys()
                    .filter(|k| prefixes.iter().any(|p| k.starts_with(p)))
                    .map(|k| (k.clone(), None)),
            )
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

pub(super) struct RowContext<'a> {
    pub entity: EntityType,
    pub head: &'a str,
    pub alias: Option<&'a str>,
    pub classifier: &'a str,
    pub factory: bool,
    pub stale: bool,
}

fn logged(
    matched: Matched,
    actions: Vec<PageAction>,
    forced: Option<&'static str>,
    outcome: RowOutcome,
) -> Resolved {
    Resolved {
        matched: Some(matched),
        mode: ControlMode::Recording,
        page_actions: actions,
        forced,
        row: Some(outcome),
        by_row: true,
        ..Default::default()
    }
}

pub(super) fn resolve_rows(
    snap: &Snapshot,
    m: &RowContext<'_>,
    claim: &Claim,
    cfg: &ControlConfig,
    safety_ok: &mut dyn FnMut(EntityType) -> bool,
) -> Option<Resolved> {
    let entity = entity_of(m.entity);
    let names: Vec<&str> = std::iter::once(m.head).chain(m.alias).collect();
    let classifier = (!m.classifier.is_empty() && valid_name(m.classifier)).then_some(m.classifier);
    let (scope_lkey, scope) = snap.mover(entity, &names, classifier)?;
    let mut outcome = RowOutcome::new(claim);
    let row_lkey = level_lkey(entity, &scope.head, scope.classifier.as_deref(), claim.row);
    let scope_matched = || Matched {
        lkey: scope_lkey.to_owned(),
        record: scope.clone(),
    };
    let record = match snap.at(&row_lkey) {
        None => return Some(logged(scope_matched(), Vec::new(), None, outcome)),
        Some(Hit::Record(_, r)) if r.mode != ControlMode::Rows => r,
        Some(_) => {
            return Some(logged(
                scope_matched(),
                Vec::new(),
                Some("unreadable"),
                outcome,
            ));
        }
    };
    let matched = Matched {
        lkey: row_lkey,
        record: record.clone(),
    };
    let entity_ok =
        record.entity_type == entity && record.actions.iter().all(|a| a.entity() == entity);
    match record.mode {
        ControlMode::Rules => {
            return Some(Resolved {
                matched: Some(matched),
                row: Some(outcome),
                ..Default::default()
            });
        }
        ControlMode::Recording => {
            let forced = if record.actions.is_empty() {
                None
            } else if m.factory {
                Some("factory_managed")
            } else if !entity_ok {
                Some("wrong_entity")
            } else {
                None
            };
            let actions = match forced {
                None => record.actions.clone(),
                Some(_) => Vec::new(),
            };
            return Some(logged(matched, actions, forced, outcome));
        }
        _ => {}
    }
    if m.factory {
        return Some(logged(
            matched,
            Vec::new(),
            Some("factory_managed"),
            outcome,
        ));
    }
    if record.actions.is_empty() || !entity_ok {
        return Some(logged(matched, Vec::new(), Some("wrong_entity"), outcome));
    }
    let actions = record.actions.clone();
    let forced = if !cfg.enabled {
        Some("kill_switch")
    } else if m.stale {
        Some("stale_snapshot")
    } else if !safety_ok(m.entity) {
        Some("safety_checks_changed")
    } else if claim.row.level().is_some() && cfg.confidence.unreadable.is_some() {
        Some("confidence_config_unreadable")
    } else {
        None
    };
    if forced.is_some() {
        return Some(logged(matched, actions, forced, outcome));
    }
    if let Some(level) = claim.row.level() {
        let evidence = record
            .evidence
            .as_deref()
            .cloned()
            .or_else(|| claim.evidence.clone());
        let supports = evidence.as_ref().and_then(|e| {
            confidence::supported_level(e, claim.now_ms, cfg.confidence.evidence_max_age_ms)
        });
        outcome.evidence_supports = Some(supports);
        if supports.is_none_or(|s| s < level) {
            let overridden = record
                .evidence_override
                .as_ref()
                .filter(|o| confidence::override_active(o, record.updated_at_ms, claim.now_ms));
            match overridden {
                Some(o) => outcome.override_by = Some(o.by.clone()),
                None => {
                    let why = if evidence.is_some() {
                        "evidence_below_floor"
                    } else {
                        "evidence_missing"
                    };
                    return Some(logged(matched, actions, Some(why), outcome));
                }
            }
        }
    }
    Some(Resolved {
        matched: Some(matched),
        mode: ControlMode::Live,
        page_actions: actions,
        row: Some(outcome),
        by_row: true,
        ..Default::default()
    })
}

pub(super) fn for_compare(rec: &ActionControlRecord) -> ActionControlRecord {
    match rec.mode {
        ControlMode::Rows => ActionControlRecord {
            mode: ControlMode::Recording,
            actions: Vec::new(),
            ..rec.clone()
        },
        _ => rec.clone(),
    }
}

pub(super) fn as_running(
    rec: ActionControlRecord,
    cfg: &ControlConfig,
    stale: bool,
    safety_ok: bool,
    factory: bool,
    now_ms: i64,
) -> ActionControlRecord {
    if rec.mode != ControlMode::Live {
        return rec;
    }
    let entity = match rec.entity_type {
        ControlEntity::User => EntityType::User,
        ControlEntity::Post => EntityType::Post,
    };
    let (mut mode, _, _) = effective(&rec, entity, factory, cfg, stale, safety_ok);
    if let Some(level) = rec.confidence.and_then(LevelRow::level) {
        let supports = rec.evidence.as_deref().and_then(|e| {
            confidence::supported_level(e, now_ms, cfg.confidence.evidence_max_age_ms)
        });
        let overridden = rec
            .evidence_override
            .as_deref()
            .is_some_and(|o| confidence::override_active(o, rec.updated_at_ms, now_ms));
        if cfg.confidence.unreadable.is_some()
            || (supports.is_none_or(|s| s < level) && !overridden)
        {
            mode = ControlMode::Recording;
        }
    }
    ActionControlRecord { mode, ..rec }
}

pub(super) fn evidence_harsher(
    before: Option<&ActionControlRecord>,
    after: &ActionControlRecord,
    now_ms: i64,
    max_age_ms: i64,
) -> Vec<String> {
    let mut why = Vec::new();
    if after.mode != ControlMode::Live {
        return why;
    }
    let supports = |r: Option<&ActionControlRecord>| {
        r.and_then(|r| r.evidence.as_deref())
            .and_then(|e| confidence::supported_level(e, now_ms, max_age_ms))
    };
    let (was, now) = (supports(before), supports(Some(after)));
    if now > was
        && let Some(l) = now
    {
        why.push(format!(
            "new evidence lets the record act at {}",
            l.as_str()
        ));
    }
    let expiry = |r: Option<&ActionControlRecord>| {
        r.and_then(|r| {
            r.evidence_override
                .as_ref()
                .filter(|o| confidence::override_active(o, r.updated_at_ms, now_ms))
                .map(|o| o.expires_at_ms)
        })
    };
    if let Some(until) = expiry(Some(after))
        && expiry(before).is_none_or(|was| until > was)
    {
        why.push(format!(
            "an override lets the level act past its evidence floor until {until} (epoch ms)"
        ));
    }
    why
}

fn row_problem(
    entity: ControlEntity,
    row: LevelRow,
    mode: ControlMode,
    actions: &[PageAction],
    evidence: Option<&ConfidenceEvidence>,
    evidence_override: Option<&EvidenceOverride>,
    now_ms: i64,
) -> Result<(), SetError> {
    let r = row.as_str();
    let bad = |code: &'static str, msg: String| Err(SetError::Invalid(code, msg));
    match mode {
        ControlMode::Rows => {
            return bad(
                "invalid_row",
                format!("the {r} row's mode is rules, recording or live"),
            );
        }
        ControlMode::Rules if !actions.is_empty() => {
            return bad(
                "rules_takes_no_actions",
                format!("the {r} row follows the GrowthBook rules; send actions: []"),
            );
        }
        ControlMode::Live if actions.is_empty() => {
            return bad(
                "live_needs_actions",
                format!("the {r} row is live and needs at least one action"),
            );
        }
        _ => {}
    }
    let mut kinds = HashSet::new();
    for a in actions {
        if !kinds.insert(a.kind()) {
            return bad(
                "duplicate_action",
                format!("{} appears more than once on the {r} row", a.kind()),
            );
        }
        if a.entity() != entity {
            return bad(
                "wrong_entity_actions",
                format!(
                    "a {} row takes only {} actions",
                    entity.as_str(),
                    entity.as_str()
                ),
            );
        }
        if !PageAction::menu(entity).contains(a) {
            return bad(
                "invalid_lifetime",
                format!(
                    "{} is not offered with that lifetime; a post label lasts 30 days or permanently",
                    a.kind()
                ),
            );
        }
    }
    if let Some(ev) = evidence {
        if row.level().is_none() {
            return bad(
                "invalid_evidence",
                "the missing row has no level, so it takes no evidence".into(),
            );
        }
        if let Some(p) = confidence::evidence_problem(ev, now_ms) {
            return bad("invalid_evidence", p);
        }
    }
    if let Some(ov) = evidence_override {
        if row.level().is_none() || mode != ControlMode::Live {
            return bad(
                "invalid_override",
                "an evidence override belongs on a live low, mid or high row".into(),
            );
        }
        if let Some(p) = confidence::override_problem(ov, now_ms) {
            return bad("invalid_override", p);
        }
    }
    Ok(())
}

pub(super) fn validate_confidence(
    req: &ActionControlPutRequest,
    now_ms: i64,
) -> Result<(), SetError> {
    let bad = |code: &'static str, msg: String| Err(SetError::Invalid(code, msg));
    if req.confidence.is_some() && req.band.is_some() {
        return bad(
            "invalid_scope",
            "a record is keyed by band or by row, not both".into(),
        );
    }
    if req.rows.is_some() && req.mode != ControlMode::Rows {
        return bad(
            "rows_only_with_mode_rows",
            "rows are sent with mode rows, the move onto rows".into(),
        );
    }
    if req.mode == ControlMode::Rows {
        if req.band.is_some() || req.confidence.is_some() {
            return bad(
                "invalid_scope",
                "a head or head + classifier moves onto rows, not a band or a row".into(),
            );
        }
        if !req.actions.is_empty() {
            return bad(
                "rows_take_no_actions",
                "mode rows acts with each row's own setting; send actions: [] and the rows in rows"
                    .into(),
            );
        }
        if req.evidence.is_some() || req.evidence_override.is_some() {
            return bad(
                "invalid_evidence",
                "evidence and overrides belong on a row".into(),
            );
        }
        if let Some(rows) = &req.rows {
            for row in LevelRow::ALL {
                if let Some(s) = rows.get(row) {
                    row_problem(
                        req.entity_type,
                        row,
                        s.mode,
                        &s.actions,
                        s.evidence.as_ref(),
                        s.evidence_override.as_ref(),
                        now_ms,
                    )?;
                }
            }
        }
        return Ok(());
    }
    match req.confidence {
        Some(row) => row_problem(
            req.entity_type,
            row,
            req.mode,
            &req.actions,
            req.evidence.as_ref(),
            req.evidence_override.as_ref(),
            now_ms,
        ),
        None if req.evidence.is_some() => bad(
            "invalid_evidence",
            "evidence belongs on a low, mid or high row".into(),
        ),
        None if req.evidence_override.is_some() => bad(
            "invalid_override",
            "an evidence override belongs on a live low, mid or high row".into(),
        ),
        None => Ok(()),
    }
}

pub(super) fn record_problem(rec: &ActionControlRecord, now_ms: i64) -> Option<&'static str> {
    let ahead = now_ms.saturating_add(confidence::CLOCK_SKEW_MS);
    if rec.updated_at_ms > ahead {
        return Some("written later than this pod's clock");
    }
    if rec
        .evidence
        .as_ref()
        .is_some_and(|ev| ev.read_at_ms > ahead)
    {
        return Some("evidence read later than this pod's clock");
    }
    if let Some(ov) = &rec.evidence_override {
        if ov.by.trim().is_empty() || ov.reason.trim().is_empty() {
            return Some("an evidence override must name a person and a reason");
        }
        if ov.expires_at_ms <= rec.updated_at_ms {
            return Some("an evidence override can't expire before its own write");
        }
        if ov.expires_at_ms
            > rec
                .updated_at_ms
                .saturating_add(confidence::EVIDENCE_OVERRIDE_MAX_MS)
        {
            return Some("an evidence override can't last past 7 days from its write");
        }
    }
    if rec.confidence.is_some() && rec.band.is_some() {
        return Some("a record can't be keyed by both band and row");
    }
    if rec.mode == ControlMode::Rows
        && (rec.band.is_some() || rec.confidence.is_some() || !rec.actions.is_empty())
    {
        return Some("mode rows is set on a head or head + classifier, with no actions");
    }
    let level = rec.confidence.and_then(LevelRow::level);
    if rec.evidence.is_some() && level.is_none() {
        return Some("evidence on a record that isn't a low, mid or high row");
    }
    if rec.evidence_override.is_some() && level.is_none() {
        return Some("an evidence override on a record that isn't a low, mid or high row");
    }
    if let Some(ev) = &rec.evidence {
        let lb = ev.precision_lower_bound;
        if !(lb.is_finite() && (0.0..=1.0).contains(&lb)) {
            return Some("evidence lower bound is not a number from 0 to 1");
        }
    }
    None
}

pub(super) fn view(cfg: &ControlConfig) -> ConfidenceView {
    ConfidenceView {
        rows: LevelRow::ALL
            .into_iter()
            .map(|row| RowView {
                row,
                floor: row.level().map(ConfidenceLevel::floor),
                suggested_actions: AllowedActions {
                    user: PageAction::suggested(ControlEntity::User, row),
                    post: PageAction::suggested(ControlEntity::Post, row),
                },
            })
            .collect(),
        evidence_max_age_ms: cfg.confidence.evidence_max_age_ms,
        config_error: cfg.confidence.unreadable.clone(),
    }
}


struct RowWrite {
    row: LevelRow,
    lkey: String,
    after: ActionControlRecord,
        prior: Option<Vec<u8>>,
}

struct Change {
    lkey: String,
    after: Option<ActionControlRecord>,
    before: Option<ActionControlRecord>,
    prior: Option<Vec<u8>>,
}

impl ActionControl {
                #[allow(clippy::too_many_arguments)]
    pub(super) async fn set_rows(
        &self,
        req: ActionControlPutRequest,
        who: &str,
        key_id: &str,
        now_ms: i64,
        cfg: &ControlConfig,
        safety: &SafetyChecksByEntity,
        seen: &[ActionControlSeen],
        deadline: Instant,
    ) -> Result<SetOutcome, SetError> {
        let entity = req.entity_type;
        let classifier = req.classifier.as_deref();
        let scope_lkey = lkey_for(entity, &req.head, classifier, None);
        let alias = cfg.aliases.get(&req.head).map(String::as_str);
        let names: Vec<&str> = std::iter::once(req.head.as_str()).chain(alias).collect();
        let rows = self.scan_rows().await?;
        let snap = Snapshot::of_rows(&rows, "");
        let scope_prior = self.get_raw(&scope_lkey).await?;
        let scope_version = scope_prior.as_deref().map(raw_version);
        let scope_before = scope_prior.as_deref().and_then(|b| parse_record(b).ok());
        let moving = req.mode == ControlMode::Rows;
        let leaving = !moving && req.confidence.is_none();
        let on_rows = scope_before
            .as_ref()
            .is_some_and(|r| r.mode == ControlMode::Rows);
        if ((moving || leaving) && scope_version != req.expected_version) || (leaving && !on_rows) {
            return Err(SetError::Conflict(scope_before, scope_version));
        }
        if moving {
            let not_rules: Vec<String> = snap
                .under_scope(entity, &names, classifier)
                .into_iter()
                .filter(|(_, mode)| *mode != Some(ControlMode::Rules))
                .map(|(k, _)| k)
                .collect();
            if !not_rules.is_empty() {
                return Err(SetError::Invalid(
                    "band_settings_not_rules",
                    format!(
                        "set these to rules before moving onto rows: {}",
                        not_rules.join(", ")
                    ),
                ));
            }
        } else if !on_rows {
            return Err(SetError::Invalid(
                "scope_not_on_rows",
                format!("{scope_lkey} is not on rows; move it with mode rows and its four rows"),
            ));
        }
        let record = |row: Option<LevelRow>,
                      mode: ControlMode,
                      actions: Vec<PageAction>,
                      evidence: Option<ConfidenceEvidence>,
                      evidence_override: Option<EvidenceOverride>,
                      version: u64| {
            let mut rec = ActionControlRecord {
                v: ACTION_CONTROL_RECORD_VERSION,
                entity_type: entity,
                head: req.head.clone(),
                classifier: req.classifier.clone(),
                band: None,
                mode,
                actions,
                reason: req.reason.trim().to_owned(),
                updated_by: who.to_owned(),
                key_id: key_id.to_owned(),
                updated_at_ms: now_ms,
                version,
                write_nonce: new_nonce(),
                confidence: row,
                evidence: evidence.map(Box::new),
                evidence_override: evidence_override.map(Box::new),
            };
            rec.v = rec.schema_version();
            rec
        };
        let targets: Vec<LevelRow> = match req.confidence {
            Some(row) => vec![row],
            None if leaving => Vec::new(),
            None => LevelRow::ALL.to_vec(),
        };
        let mut writes: Vec<RowWrite> = Vec::with_capacity(targets.len());
        let mut befores: Vec<Option<ActionControlRecord>> = Vec::with_capacity(targets.len());
        for row in targets {
            let lkey = level_lkey(entity, &req.head, classifier, row);
            let prior = self.get_raw(&lkey).await?;
            let version = prior.as_deref().map(raw_version);
            let before = prior.as_deref().and_then(|b| parse_record(b).ok());
            let (setting, expected) = if moving {
                let s = req.rows.as_ref().and_then(|rows| rows.get(row));
                (
                    s.map(|s| {
                        (
                            s.mode,
                            s.actions.clone(),
                            s.evidence.clone(),
                            s.evidence_override.clone(),
                        )
                    }),
                    s.and_then(|s| s.expected_version),
                )
            } else {
                (
                    Some((
                        req.mode,
                        req.actions.clone(),
                        req.evidence.clone(),
                        req.evidence_override.clone(),
                    )),
                    req.expected_version,
                )
            };
            if expected != version {
                return Err(SetError::Conflict(before, version));
            }
            let (mode, actions, evidence, evidence_override) =
                setting.unwrap_or((ControlMode::Recording, Vec::new(), None, None));
            writes.push(RowWrite {
                row,
                lkey,
                after: record(
                    Some(row),
                    mode,
                    actions,
                    evidence,
                    evidence_override,
                    version.unwrap_or(0) + 1,
                ),
                prior,
            });
            befores.push(before);
        }

        let recording = record(None, ControlMode::Recording, Vec::new(), None, None, 0);
        let has_set_aside = !snap.under_scope(entity, &names, classifier).is_empty();
        let set_aside = has_set_aside && snap.mover(entity, &names, classifier).is_none();
        let scope_write = (moving || leaving).then(|| {
            let (mode, actions) = if moving {
                (ControlMode::Rows, Vec::new())
            } else {
                (req.mode, req.actions.clone())
            };
            record(
                None,
                mode,
                actions,
                None,
                None,
                scope_version.unwrap_or(0) + 1,
            )
        });
        let rules_after = record(None, ControlMode::Rules, Vec::new(), None, None, 0);
        let mut compare: Vec<(LevelRow, ActionControlRecord)> =
            writes.iter().map(|w| (w.row, w.after.clone())).collect();
        if let (true, Some(after)) = (leaving, &scope_write) {
            let without = Snapshot::of_rows(&rows, &scope_lkey);
            for row in LevelRow::ALL {
                let next = match after.mode {
                    ControlMode::Rules => without
                        .row_decider(entity, &names, classifier, row, &recording)
                        .unwrap_or_else(|| rules_after.clone()),
                    _ => after.clone(),
                };
                compare.push((row, next));
                if has_set_aside && after.mode != ControlMode::Rules {
                    compare.push((row, rules_after.clone()));
                }
            }
        }
        let safety_ok = match entity {
            ControlEntity::User => safety.user.ok,
            ControlEntity::Post => safety.post.ok,
        };
        let factory = seen
            .iter()
            .any(|s| s.factory && s.head == req.head && s.entity_type == entity);
        let stale = self.is_stale();
        let rules_specs: Option<Vec<Value>> = {
            let scope_rows: Vec<&ActionControlSeen> = seen
                .iter()
                .filter(|s| s.head == req.head && s.entity_type == entity)
                .filter(|s| classifier.is_none_or(|c| s.classifier == c))
                .collect();
            (!scope_rows.is_empty()).then(|| {
                scope_rows
                    .iter()
                    .flat_map(|s| s.action_specs.as_array().cloned().unwrap_or_default())
                    .collect()
            })
        };
        let mut harsher: Vec<String> = Vec::new();
        for (row, after) in compare {
            let mut priors = vec![snap.row_decider(entity, &names, classifier, row, &recording)];
            if set_aside {
                priors.push(None);
            }
            let judged_priors = priors
                .into_iter()
                .map(|prior| prior.map(|p| as_running(p, cfg, stale, safety_ok, factory, now_ms)));
            for judged in judged_priors {
                let why = harsher_because(judged.as_ref(), &after, rules_specs.as_deref())
                    .into_iter()
                    .chain(evidence_harsher(
                        judged.as_ref(),
                        &after,
                        now_ms,
                        cfg.confidence.evidence_max_age_ms,
                    ));
                for reason in why {
                    let reason = format!("{} row: {reason}", row.as_str());
                    if !harsher.contains(&reason) {
                        harsher.push(reason);
                    }
                }
            }
        }
        if !harsher.is_empty() && req.confirm.as_deref() != Some(CONFIRM_WORD) {
            return Err(SetError::Invalid(
                "confirm_required",
                format!(
                    "this change enforces more ({}); send confirm: \"{CONFIRM_WORD}\"",
                    harsher.join("; ")
                ),
            ));
        }

        let removals = LevelRow::ALL
            .into_iter()
            .filter(|_| leaving)
            .filter_map(|row| {
                let lkey = level_lkey(entity, &req.head, classifier, row);
                let (_, prior) = rows.iter().find(|(k, _)| *k == lkey)?;
                Some(Change {
                    before: parse_record(prior).ok(),
                    prior: Some(prior.clone()),
                    after: None,
                    lkey,
                })
            });
        let changes: Vec<Change> = writes
            .iter()
            .zip(&befores)
            .map(|(w, before)| Change {
                lkey: w.lkey.clone(),
                after: Some(w.after.clone()),
                before: before.clone(),
                prior: w.prior.clone(),
            })
            .chain(scope_write.as_ref().map(|rec| Change {
                lkey: scope_lkey.clone(),
                after: Some(rec.clone()),
                before: scope_before.clone(),
                prior: scope_prior.clone(),
            }))
            .chain(removals)
            .collect();
        let values = changes
            .iter()
            .map(|c| c.after.as_ref().map(serde_json::to_vec).transpose())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SetError::Store(e.to_string()))?;
        if Instant::now() + self.scan_timeout * (2 * changes.len()) as u32 > deadline {
            return Err(SetError::Store(
                "the save took longer than its hold on the head; nothing was written, reload and save again"
                    .into(),
            ));
        }
        for (i, (c, value)) in changes.iter().zip(values).enumerate() {
            if let Err(e) = self.write_raw(&c.lkey, value, deadline).await {
                return Err(self.put_back(&changes[..=i], e, deadline).await);
            }
        }
        let mut history_written = true;
        for c in &changes {
            if let Some(rec) = &c.after {
                history_written &= self
                    .write_history(&c.lkey, rec, c.before.as_ref(), key_id, now_ms)
                    .await;
            }
        }
        for c in &changes {
            match &c.after {
                Some(rec) => self.read_back(&c.lkey, rec).await?,
                None => self.read_back_removed(&c.lkey).await?,
            }
        }
        for c in changes {
            match c.after {
                Some(rec) => self.apply_local(c.lkey, rec),
                None => self.forget_local(&c.lkey),
            }
        }
        info!(
            lkey = scope_lkey,
            moved = moving,
            rows = writes.len(),
            updated_by = %who,
            key_id,
            "action control rows written"
        );
        let factory_heads: HashSet<(ControlEntity, String)> = seen
            .iter()
            .filter(|s| s.factory)
            .map(|s| (s.entity_type, s.head.clone()))
            .collect();
        let (lkey, rec, before) = match scope_write {
            Some(rec) => (scope_lkey, rec, scope_before),
            None => {
                let w = writes.remove(0);
                (w.lkey, w.after, befores.remove(0))
            }
        };
        let view = self.view(&lkey, &rec, cfg, stale, safety, &factory_heads);
        Ok(SetOutcome {
            view,
            before,
            harsher_because: harsher,
            history_written,
        })
    }

                                            async fn put_back(&self, written: &[Change], err: SetError, deadline: Instant) -> SetError {
        let SetError::Store(msg) = err else {
            return err;
        };
        for c in written.iter().rev() {
            if self
                .write_raw(&c.lkey, c.prior.clone(), deadline)
                .await
                .is_err()
            {
                return SetError::Store(format!(
                    "{msg}; putting {} back failed, so the save may have applied: the records \
                     before it keep what this save confirmed; reload to see what the store holds",
                    c.lkey
                ));
            }
        }
        SetError::Store(format!(
            "{msg}; every record this save wrote was put back as it was, so nothing changed"
        ))
    }

        async fn write_history(
        &self,
        lkey: &str,
        rec: &ActionControlRecord,
        before: Option<&ActionControlRecord>,
        key_id: &str,
        now_ms: i64,
    ) -> bool {
        let entry = ActionControlHistoryEntry {
            at_ms: now_ms,
            version: rec.version,
            lkey: lkey.to_owned(),
            scope: rec.scope().to_owned(),
            updated_by: rec.updated_by.clone(),
            key_id: key_id.to_owned(),
            reason: rec.reason.clone(),
            env: self.env.clone(),
            before: before.cloned(),
            after: rec.clone(),
        };
        let Ok(v) = serde_json::to_vec(&entry) else {
            return false;
        };
        let pkey = format!("{HISTORY_PKEY_PREFIX}{}", rec.head);
        let hkey = format!("{now_ms:013}-{:010}-{lkey}", rec.version);
        let put = self.store.put(&pkey, &hkey, v, HISTORY_TTL_SECS);
        match tokio::time::timeout(self.scan_timeout, put).await {
            Ok(Ok(())) => true,
            Ok(Err(e)) => {
                warn!(error = %e, lkey, "action control: history write failed");
                false
            }
            Err(_) => {
                warn!(lkey, "action control: history write timed out");
                false
            }
        }
    }

            async fn read_back_removed(&self, lkey: &str) -> Result<(), SetError> {
        match self.get_record(lkey).await {
            Ok(None) => Ok(()),
            Ok(Some((r, _))) => Err(SetError::LostRace(r)),
            Err(_) => Err(SetError::Store(
                "the write landed but reading it back failed; pods pick up whatever the store holds within 5 s, so reload to see it"
                    .into(),
            )),
        }
    }

            async fn read_back(&self, lkey: &str, rec: &ActionControlRecord) -> Result<(), SetError> {
        match self.get_record(lkey).await {
            Ok(Some((Some(r), _))) if r.write_nonce == rec.write_nonce => Ok(()),
            Ok(Some((Some(r), _))) => Err(SetError::LostRace(Some(r))),
            Ok(_) | Err(_) => Err(SetError::Store(
                "the write landed but reading it back failed; pods pick up whatever the store holds within 5 s, so reload to see it"
                    .into(),
            )),
        }
    }
}
