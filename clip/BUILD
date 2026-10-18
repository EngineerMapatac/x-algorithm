python3_library(dependencies = [
    ":deps_v100_or_cpu",
    ":source",
])

python3_library(
    name = "clip_a100",
    dependencies = [
        ":deps_a100",
        ":source",
    ],
    sources = [
        "*.py",
        "datasets/*.py",
        "utils/*.py",
    ],
    tags = [
        "no-mypy",
    ],
)

python3_library(
    name = "source",
    sources = [
        "*.py",
        "datasets/*.py",
        "utils/*.py",
    ],
)

python3_library(
    name = "deps_a100",
    dependencies = [
        ":deps_common",
        "3rdparty/python/torch:1_9_1_cuda",
        "3rdparty/python/torchvision:0_10_1_cuda",
    ],
    sources = [],
)

python3_library(
    name = "deps_v100_or_cpu",
    dependencies = [
        ":deps_common",
        "3rdparty/python/torch:1_9_0_hashfix",
        "3rdparty/python/torchvision:0_10_0_hashfix",
    ],
    sources = [],
)

python3_library(
    name = "deps_common",
    dependencies = [
        "3rdparty/python/clip-by-openai:default",
        "3rdparty/python/faiss-gpu:default",
        "3rdparty/python/google-cloud-bigquery:default",
        "3rdparty/python/google-cloud-storage",
        "3rdparty/python/h5py:3_1_0",
        "3rdparty/python/numpy:default",
        "3rdparty/python/onnx:default",
        "3rdparty/python/onnx-tf:default",
        "3rdparty/python/pyarrow:2_0_0",
        "3rdparty/python/pyhive:default",
        "3rdparty/python/tensorflow:2_5_0",
        "3rdparty/python/tensorflow-models-official-resnet:default",
        "3rdparty/python/transformers:default",
        "magicpony/common/file_access/src/python/twitter/magicpony/common/file_access",
        "media-understanding/common/src/main/python/com/twitter/media_understanding/common/dbv2_utils",
        "media-understanding/image-classification/src/main/python/twitter/image_classification:source",
        "src/python/twitter/deepbird:deepbird-tensorflow-2",
        "src/python/twitter/deepbird/data",
        "src/python/twitter/deepbird/image",
        "src/python/twitter/ml/common:hadoop",
        "src/python/twitter/ml/tfx/utils",
    ],
    sources = [],
)

python37_binary(
    name = "clip_training",
    dependencies = [
        ":clip",
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip:clip_training",
    ],
    source = "training.py",
    tags = ["no-mypy"],
)

python37_binary(
    name = "clip_training_a100",
    dependencies = [
        ":clip_a100",
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip:clip_training_a100",
    ],
    source = "training.py",
)

python37_binary(
    name = "clip_train_data_notebook",
    dependencies = [
        ":clip",
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip:clip_train_data_notebook",
    ],
    entry_point = "twitter.jupyter.notebook.pex:launcher",
)

python37_binary(
    name = "clip_clustering_notebook",
    dependencies = [
        ":deps_v100_or_cpu",
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip:clip_clustering_notebook",
    ],
    entry_point = "twitter.jupyter.notebook.pex:launcher",
    tags = ["no-mypy"],
)
