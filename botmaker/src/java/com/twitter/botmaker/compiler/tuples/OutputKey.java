package com.twitter.botmaker.compiler.tuples;

import com.google.common.collect.ImmutableList;

public class OutputKey extends StructTuple {

  public static final ImmutableList<String> FIELD_NAMES = ImmutableList.of(
      "namespace",
      "typeName",
      "name"
  );

  public OutputKey(
      String namespace,
      String typeName,
      String name) {
    super(FIELD_NAMES, ImmutableList.of(namespace, typeName, name));
  }
}
