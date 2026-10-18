python3_library(
    name = "source",
    sources = ["*.py"],
)

python37_binary(
    name = "export_dbv2_image_model",
    dependencies = [
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip/scripts:export_dbv2_image_model",
        "media-understanding/clip/src/main/python/twitter/clip",
    ],
    source = "export_dbv2_image_model.py",
    tags = ["no-mypy"],
)

python37_binary(
    name = "export_keras_models",
    dependencies = [
        "3rdparty/python/_closures/media-understanding/clip/src/main/python/twitter/clip/scripts:export_keras_models",
        "media-understanding/clip/src/main/python/twitter/clip",
    ],
    source = "export_keras_models.py",
    tags = ["no-mypy"],
)
