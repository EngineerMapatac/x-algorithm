import random

import tensorflow as tf


def class_func(features, label):
    return label


resample_fn = tf.data.experimental.rejection_resample(
    class_func, target_dist=[0.5, 0.5], seed=0
)


def decode_fn_embedding(example_proto, embedding_dim):
    feature_description = {
        "media_embedding": tf.io.FixedLenFeature([embedding_dim], dtype=tf.float32),
        "labels": tf.io.FixedLenFeature([], dtype=tf.int64),
    }

    example = tf.io.parse_single_example(example_proto, feature_description)

    return example


def preprocess_embedding_example(
    example_dict, positive_label=1, features_as_dict=False
):
    labels = example_dict["labels"]
    label = tf.math.reduce_any(labels == positive_label)
    label = tf.cast(label, tf.int32)
    embedding = example_dict["media_embedding"]

    if features_as_dict:
        features = {"media_embedding": embedding}
    else:
        features = embedding

    return features, label


def build_dataset(
    dataset_path, embedding_dim, batch_size, do_resample=False, do_repeat=False
):
    file_pattern = f"{dataset_path}/*.tfrecord"
    files = tf.io.gfile.glob(file_pattern)

    random.shuffle(files)

    if not len(files):
        raise ValueError(f"Did not find any files matching {file_pattern}")

    ds = tf.data.TFRecordDataset(files).map(
        lambda x: decode_fn_embedding(x, embedding_dim)
    )
    ds = ds.map(lambda x: preprocess_embedding_example(x, positive_label=1))

    if do_resample:
        ds = ds.apply(resample_fn).map(lambda _, b: (b))

    ds = ds.batch(batch_size=batch_size)

    if do_repeat:
        ds = ds.shuffle(buffer_size=10).repeat()

    return ds
