scala_library(
    compiler_option_sets = ["fatal_warnings"],
    dependencies = [
        "3rdparty/jvm/commons-io",
        "iago-internal/src/main/scala/com/twitter/iago/internal",
        "iago-internal/src/main/scala/com/twitter/iago/internal/legacy",
        "iago/src/main/scala/com/twitter/iago",
        "iago/src/main/scala/com/twitter/iago/legacy",
        "media-understanding/media-model-proxy/src/main/thrift:thrift-scala",
        "twitter-server/server/src/main/scala",
    ],
    sources = ["MediaModelProxyLoadTest.scala"],
)

scala_library(
    name = "sampler",
    compiler_option_sets = ["fatal_warnings"],
    dependencies = [
        "3rdparty/src/jvm/com/twitter/scalding:args",
        "3rdparty/src/jvm/com/twitter/scalding:commons",
        "3rdparty/src/jvm/com/twitter/scalding:core",
        "3rdparty/src/jvm/com/twitter/scalding:date",
        "3rdparty/src/jvm/com/twitter/scalding:db",
        "media-understanding/media-model-proxy/src/main/thrift:thrift-scala",
        "src/scala/com/twitter/scalding_internal/args",
        "src/scala/com/twitter/scalding_internal/dalv2",
        "src/scala/com/twitter/scalding_internal/error_handling",
        "src/scala/com/twitter/scalding_internal/job",
        "src/scala/com/twitter/scalding_internal/job/analytics_batch",
        "src/scala/com/twitter/scalding_internal/source",
        "src/thrift/com/twitter/dal:dal-java",
        "src/thrift/com/twitter/servo:servo-exception-java",
        "twadoop_config/configuration/log_categories/group/cortex:cortex_media_annotator-scala",
        "twitter-server-internal",
    ],
    platform = "java8",
    sources = ["MediaModelProxyLogSampler.scala"],
    tags = ["bazel-only"],
)

hadoop_binary(
    name = "media-model-proxy-sampler-execution-app-deploy",
    dependencies = [
        ":sampler",
    ],
    main = "com.twitter.media_understanding.media_model_proxy.loadtest.MediaModelProxyUniformSamplerJob",
    platform = "java8",
    runtime_platform = "java11",
    tags = [
        "bazel-compatible",
        "bazel-compatible:migrated",
        "bazel-only",
    ],
)
