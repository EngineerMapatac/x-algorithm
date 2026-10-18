create_thrift_libraries(
    base_name = "app",
    dependency_roots = [
        "botmaker/src/thrift/com/twitter/botmaker/rule",
        "botmaker/src/thrift/com/twitter/botmaker/runtime",
        "finatra-internal/thrift/src/main/thrift",
        "macaw-login/knowndevices/thrift/src/main/thrift",
        "safety_systems/simple_list/src/thrift/com/twitter/safety_systems/simple_list:thrift",
        "src/thrift/com/twitter/botmaker",
        "src/thrift/com/twitter/relevance/feature_store",
        "src/thrift/com/twitter/service/scarecrow/gen:tiered-actions",
    ],
    generate_languages = [
        "java",
        "python",
        "scala",
        "strato",
    ],
    platform = "java8",
    provides_java_name = "app-thrift-java",
    provides_python_name = "app-thrift-python",
    provides_scala_name = "app-thrift-scala",
    sources = [
        "app.thrift",
    ],
    tags = ["bazel-compatible"],
)
