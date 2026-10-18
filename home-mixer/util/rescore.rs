use rand::Rng;

pub fn rescore_slots(
    original_scores: &[f64],
    effective_scores: &[f64],
    is_anchor: &[bool],
    is_treatment: &[bool],
) -> Vec<f64> {
    let n = original_scores.len();
    let mut result: Vec<f64> = original_scores.to_vec();

    let participants: Vec<usize> = (0..n).filter(|&i| !is_anchor[i]).collect();
    if participants.is_empty() {
        return result;
    }

    let mut by_t0 = participants.clone();
    by_t0.sort_by(|&a, &b| original_scores[b].total_cmp(&original_scores[a]));
    let mut counterfactual_t0 = vec![0usize; n];
    for (rank, &i) in by_t0.iter().enumerate() {
        counterfactual_t0[i] = rank;
    }

    let mut by_t1 = participants.clone();
    by_t1.sort_by(|&a, &b| effective_scores[b].total_cmp(&effective_scores[a]));
    let mut counterfactual_t1 = vec![0usize; n];
    for (rank, &i) in by_t1.iter().enumerate() {
        counterfactual_t1[i] = rank;
    }

    let mut rng = rand::rng();
    let mut unified: Vec<(usize, usize, u64)> = participants
        .iter()
        .map(|&i| {
            let counterfactual_rank = if is_treatment[i] {
                counterfactual_t1[i]
            } else {
                counterfactual_t0[i]
            };
            (i, counterfactual_rank, rng.random::<u64>())
        })
        .collect();
    unified.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));

    let mut slots: Vec<f64> = participants.iter().map(|&i| original_scores[i]).collect();
    slots.sort_by(|a, b| b.total_cmp(a));
    for (slot_idx, &(i, _, _)) in unified.iter().enumerate() {
        result[i] = slots[slot_idx];
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_anchors_returns_unchanged() {
        let scores = vec![100.0, 80.0, 60.0];
        let result = rescore_slots(
            &scores,
            &scores,
            &[true, true, true],
            &[false, false, false],
        );
        assert!((result[0] - 100.0).abs() < 1e-9);
        assert!((result[1] - 80.0).abs() < 1e-9);
        assert!((result[2] - 60.0).abs() < 1e-9);
    }

    #[test]
    fn rank_pair_gets_the_two_slots() {
        let original = vec![100.0, 90.0, 80.0, 70.0];
        let effective = vec![100.0, 90.0, 150.0, 70.0];
        let anchors = [true, false, false, true];
        let treatment = [false, false, true, false];
        let result = rescore_slots(&original, &effective, &anchors, &treatment);
        assert!((result[0] - 100.0).abs() < 1e-9);
        assert!((result[3] - 70.0).abs() < 1e-9);
        let pair: Vec<f64> = vec![result[1], result[2]];
        assert!(pair.contains(&90.0));
        assert!(pair.contains(&80.0));
    }

    #[test]
    fn treatment_only_ranks_by_effective() {
        let original = vec![90.0, 80.0, 70.0];
        let effective = vec![50.0, 100.0, 60.0];
        let anchors = [false, false, false];
        let treatment = [true, true, true];
        let result = rescore_slots(&original, &effective, &anchors, &treatment);
        assert!((result[1] - 90.0).abs() < 1e-9);
        assert!((result[2] - 80.0).abs() < 1e-9);
        assert!((result[0] - 70.0).abs() < 1e-9);
    }

    #[test]
    fn control_only_ranks_by_original() {
        let original = vec![90.0, 80.0, 70.0];
        let effective = vec![50.0, 100.0, 60.0];
        let anchors = [false, false, false];
        let treatment = [false, false, false];
        let result = rescore_slots(&original, &effective, &anchors, &treatment);
        assert!((result[0] - 90.0).abs() < 1e-9);
        assert!((result[1] - 80.0).abs() < 1e-9);
        assert!((result[2] - 70.0).abs() < 1e-9);
    }

    #[test]
    fn anchors_never_displaced() {
        let original = vec![50.0, 40.0, 30.0];
        let effective = vec![50.0, 200.0, 30.0];
        let result = rescore_slots(
            &original,
            &effective,
            &[true, false, false],
            &[false, true, false],
        );
        assert!((result[0] - 50.0).abs() < 1e-9);
        let pair: Vec<f64> = vec![result[1], result[2]];
        assert!(pair.contains(&40.0));
        assert!(pair.contains(&30.0));
    }

    #[test]
    fn uneven_groups_no_tail_pinning() {
        let original = vec![100.0, 80.0, 60.0, 40.0];
        let effective = vec![100.0, 90.0, 70.0, 50.0];
        let anchors = [false, false, false, false];
        let treatment = [false, true, true, true];
        let result = rescore_slots(&original, &effective, &anchors, &treatment);
        assert!((result[0] - 100.0).abs() < 1e-9);
        assert!((result[1] - 80.0).abs() < 1e-9);
        assert!((result[2] - 60.0).abs() < 1e-9);
        assert!((result[3] - 40.0).abs() < 1e-9);
    }

    #[test]
    fn union_set_ranks_against_full_slate() {
        let original = vec![0.9, 0.4, 0.8, 0.3];
        let effective = vec![0.2, 0.5, 0.95, 0.7];
        let anchors = [false, false, false, false];
        let treatment = [false, false, true, true];
        let result = rescore_slots(&original, &effective, &anchors, &treatment);
        let top: Vec<f64> = vec![result[0], result[2]];
        assert!(top.contains(&0.9));
        assert!(top.contains(&0.8));
        assert!((result[3] - 0.4).abs() < 1e-9);
        assert!((result[1] - 0.3).abs() < 1e-9);
    }
}
