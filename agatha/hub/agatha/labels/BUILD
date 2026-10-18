scala_library(
    name = "label-util",
    dependencies = [
        "3rdparty/src/jvm/com/twitter/scalding:core",
        "hub/agatha/src/main/thrift/com/twitter/hub/agatha:agatha-scala",
        "src/scala/com/twitter/scalding_internal/source",
        "src/scala/com/twitter/scalding_internal/source/lzo_scrooge",
    ],
    platform = "java8",
    sources = [],
    strict_deps = False,
    tags = ["bazel-compatible"],
)

scala_library(
    name = "label-generator",
    dependencies = [
        "3rdparty/src/jvm/com/twitter/scalding:core",
        "hub/agatha/src/main/thrift/com/twitter/hub/agatha:agatha-scala",
    ],
    platform = "java8",
    sources = ["LabelGenerator.scala"],
    strict_deps = False,
    tags = ["bazel-compatible"],
)

scala_library(
    dependencies = [
        ":label-generator",
        "3rdparty/src/jvm/com/twitter/scalding:core",
        "src/scala/com/twitter/scalding_internal/job",
        "src/scala/com/twitter/scalding_internal/job/analytics_batch",
    ],
    platform = "java8",
    sources = ["LabelManager.scala"],
    strict_deps = False,
    tags = ["bazel-compatible"],
)
