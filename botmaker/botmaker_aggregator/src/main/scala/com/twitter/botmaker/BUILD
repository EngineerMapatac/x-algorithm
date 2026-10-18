scala_library(
    name = "botmaker",
    dependencies = [
        "3rdparty/jvm/com/google/guava",
        "3rdparty/jvm/com/twitter/algebird:core",
        "3rdparty/jvm/com/twitter/bijection:core",
        "3rdparty/jvm/org/apache/thrift:libthrift",
        "botmaker/src/java/com/twitter/botmaker",
        "botmaker/src/scala/com/twitter/botmaker",
        "botmaker/src/thrift/com/twitter/botmaker/runtime:runtime-java",
        "src/scala/com/twitter/algebird_internal/injection",
        "src/thrift/com/twitter/algebird_internal:algebird_internal-scala",
    ],
    platform = "java8",
    sources = ["**/*.scala"],
    tags = ["bazel-compatible"],
)
