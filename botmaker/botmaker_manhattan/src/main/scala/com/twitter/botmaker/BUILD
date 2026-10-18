scala_library(
    name = "botmaker",
    dependencies = [
        "3rdparty/jvm/com/google/guava",
        "3rdparty/jvm/com/twitter/bijection:core",
        "botmaker/src/java/com/twitter/botmaker",
        "botmaker/src/scala/com/twitter/botmaker",
        "botmaker/src/thrift/com/twitter/botmaker/runtime:runtime-java",
        "stitch/stitch-core",
        "storage/clients/manhattan/client/src/main/scala",
    ],
    platform = "java8",
    sources = ["**/*.scala"],
    tags = ["bazel-compatible"],
)
