create_thrift_libraries(
    base_name = "runtime",
    dependency_roots = [
        "src/thrift/com/twitter/relevance/feature_store",
    ],
    generate_languages = [
        "java",
        "python",
        "scala",
        "strato",
    ],
    platform = "java8",
    provides_java_name = "runtime-thrift-java",
    provides_python_name = "runtime-thrift-python",
    provides_scala_name = "runtime-thrift-scala",
    sources = [
        "runtime.thrift",
    ],
    tags = ["bazel-compatible"],
)
