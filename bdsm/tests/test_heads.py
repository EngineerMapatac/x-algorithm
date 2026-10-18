import heads


def test_indices_contiguous_and_unique():
    assert sorted(h.index for h in heads.HEADS) == list(range(heads.N_HEADS))


def test_legitimate_user_is_last_head():
    assert heads.LEGITIMATE_USER_INDEX == heads.N_HEADS - 1


def test_eight_heads():
    assert heads.N_HEADS == 8
    assert len(heads.POSITIVE_HEAD_NAMES) == 7


def test_flywheel_labels_unique_across_heads():
    seen = {}
    for h in heads.HEADS:
        for label in h.flywheel_labels:
            assert label not in seen, f"{label} mapped to both {seen[label]} and {h.name}"
            seen[label] = h.name


def test_registry_hash_pinned():
    assert heads.registry_hash() == "1bb89df6b629a615"
    assert len(heads.registry_hash()) == 16


def test_dummy_inference_labels_are_eight_not_v4_twelve():
    import batch_prefetcher

    assert batch_prefetcher.N_MODEL_LABELS == heads.N_HEADS == 8
