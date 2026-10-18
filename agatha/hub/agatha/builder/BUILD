scala_library(
    dependencies = [
        "3rdparty/src/jvm/com/twitter/scalding:core",
        "hub/agatha/src/main/scala/com/twitter/hub/agatha",
        "hub/agatha/src/main/scala/com/twitter/hub/agatha/labels",
        "hub/agatha/src/main/scala/com/twitter/hub/agatha/util",
        "hub/agatha/src/main/thrift/com/twitter/hub/agatha:agatha-scala",
        "hub/pmi/src/main/scala/com/twitter/hub/pmi",
    ],
    platform = "java8",
    sources = ["*.scala"],
    strict_deps = False,
    tags = ["bazel-compatible"],
)

hadoop_binary(
    name = "builder-repl-deploy",
    basename = "builder-repl-deploy",
    dependencies = [
        ":builder",
        "science/scalding/scripts:scalding-repl-deps",
    ],
    main = "com.twitter.scalding_internal.repl.TwitterScaldingShell",
    platform = "java8",
    runtime_platform = "java11",
    tags = [
        "bazel-compatible",
        "bazel-compatible:migrated",
        "bazel-only",
    ],
)
