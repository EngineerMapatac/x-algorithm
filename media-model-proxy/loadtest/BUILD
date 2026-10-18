target(
    dependencies = [
        "media-understanding/media-model-proxy/loadtest/src/scala/com/twitter/media_understanding/media_model_proxy/loadtest",
    ],
)

jvm_binary(
    name = "bin",
    basename = "media-model-proxy-loadtest",
    dependencies = [
        ":loadtest",
    ],
    main = "com.twitter.iago.internal.ServerMain",
    runtime_platform = "java11",
)

jvm_app(
    name = "bundle",
    basename = "media-model-proxy-loadtest",
    binary = ":bin",
    bundles = [
        bundle(fileset = [
            "config/*",
            "logs/**/*",
        ]),
    ],
)
