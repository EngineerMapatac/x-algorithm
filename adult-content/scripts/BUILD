python3_library(
    name = "common-deps",
    dependencies = [
        "3rdparty/python/tensorflow:2_5_0",
    ],
)

python37_binary(
    name = "train-model",
    dependencies = [
        ":common-deps",
        "3rdparty/python/_closures/media-understanding/xx-nsfw/src/main/python/twitter/adult_content/scripts:train-model",
        "media-understanding/xx-nsfw/src/main/python/twitter/adult_content:source",
    ],
    source = "train_model.py",
)

python37_binary(
    name = "calibrate-model",
    dependencies = [
        ":common-deps",
        "3rdparty/python/_closures/media-understanding/xx-nsfw/src/main/python/twitter/adult_content/scripts:calibrate-model",
        "3rdparty/python/pandas:default",
        "3rdparty/python/scikit-learn:default",
        "media-understanding/xx-nsfw/src/main/python/twitter/adult_content:source",
    ],
    source = "calibrate_model_scores.py",
)
