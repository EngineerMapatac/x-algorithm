scala_library(
    name = "botmaker",
    dependencies = [
        "3rdparty/jvm/com/google/guava",
        "3rdparty/jvm/io/netty:netty4-tcnative-boringssl-static",
        "botmaker/src/java/com/twitter/botmaker",
        "botmaker/src/scala/com/twitter/botmaker",
        "botmaker/src/thrift/com/twitter/botmaker/runtime:runtime-java",
        "finagle-internal/mtls/src/main/scala/com/twitter/finagle/mtls/authentication",
        "finagle-internal/mtls/src/main/scala/com/twitter/finagle/mtls/client",
        "finagle/finagle-thrift/src/main/java",
        "finagle/finagle-thrift/src/main/scala",
        "finagle/finagle-thriftmux/src/main/scala",
        "src/thrift/org/apache/scribe:scribe_client-0.1.2",
    ],
    platform = "java8",
    sources = ["**/*.scala"],
    tags = ["bazel-compatible"],
)
