# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import dataclasses

import jax.tree_util


def keystr(keys):
    return ".".join(
        str(dataclasses.astuple(e)[0] if dataclasses.is_dataclass(e) else e.__getstate__()[0])
        for e in keys
    )


def tree_to_dict(tree, is_leaf=None, keep_none=True):
    if keep_none:
        assert is_leaf is None
        is_leaf = lambda t: t is None
    return {
        keystr(path): value
        for path, value in jax.tree_util.tree_flatten_with_path(tree, is_leaf=is_leaf)[0]
        if path
    }


def unflatten_dict(flat_dict, sep="."):
    result = {}
    for key, value in flat_dict.items():
        parts = key.split(sep)
        d = result
        for part in parts[:-1]:
            d = d.setdefault(part, {})
        d[parts[-1]] = value
    return result


def tree_from_dict(flat_dict):
    return unflatten_dict(flat_dict)


def has_subtree(name, flat_dict):
    if name in flat_dict:
        return True
    components = name.split(".")
    while len(components) > 1:
        components.pop()
        if flat_dict.get(".".join(components), False) is None:
            return True
    return False


def tree_map_overlap(update_leaf_fn, tree1, tree2):
    leaves1_with_paths, treedef1 = jax.tree_util.tree_flatten_with_path(tree1)
    leaves2_with_paths, treedef2 = jax.tree_util.tree_flatten_with_path(tree2)

    paths1 = {str(path): leaf for path, leaf in leaves1_with_paths}
    paths2 = {str(path): leaf for path, leaf in leaves2_with_paths}

    overlapping_paths = set(paths1.keys()).intersection(set(paths2.keys()))

    updated_overlapping_leaves = {
        path: update_leaf_fn(paths1[path], paths2[path]) for path in overlapping_paths
    }

    def update_tree(tree, updated_leaves, treedef):
        leaves, _ = jax.tree_util.tree_flatten_with_path(tree)
        updated_tree_leaves = []
        for path, leaf in leaves:
            path_str = str(path)
            if path_str in updated_leaves:
                updated_tree_leaves.append((path, updated_leaves[path_str]))
            else:
                updated_tree_leaves.append((path, leaf))
        return jax.tree_util.tree_unflatten(treedef, [leaf for _, leaf in updated_tree_leaves])

    new_tree1 = update_tree(tree1, updated_overlapping_leaves, treedef1)
    new_tree2 = update_tree(tree2, updated_overlapping_leaves, treedef2)
    return new_tree1, new_tree2


def local_nbytes(tree):
    nbytes = 0
    for array in jax.tree_util.tree_flatten(tree)[0]:
        for shard in array.addressable_shards:
            nbytes += shard.data.nbytes
    return nbytes
