create_thrift_libraries(
    base_name = "thrift",
    dependency_roots = [
        "finatra-internal/thrift/src/main/thrift",
        "mediaservices/commons/src/main/thrift",
        "src/thrift/com/twitter/ml/api:data",
    ],
    generate_languages = [
        "go",
        "java",
        "scala",
        "strato",
    ],
    platform = "java8",
    provides_java_name = "media-model-proxy-thrift-java",
    provides_scala_name = "media-model-proxy-thrift-scala",
    sources = ["**/*.thrift"],
    tags = ["bazel-compatible"],
)
