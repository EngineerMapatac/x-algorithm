scala_library(
    name = "botmaker",
    dependencies = [
        "3rdparty/jvm/com/google/guava",
        "botmaker/src/java/com/twitter/botmaker",
        "botmaker/src/scala/com/twitter/botmaker",
        "botmaker/src/thrift/com/twitter/botmaker/runtime:runtime-java",
        "finagle/finagle-memcached/src/main/scala",
    ],
    platform = "java8",
    sources = ["**/*.scala"],
    tags = ["bazel-compatible"],
)
