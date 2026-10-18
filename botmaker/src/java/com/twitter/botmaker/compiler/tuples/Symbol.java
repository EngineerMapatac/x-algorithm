package com.twitter.botmaker.compiler.tuples;

import java.util.List;

import com.google.common.collect.ImmutableList;

public class Symbol extends StructTuple {

  public static final ImmutableList<String> FIELD_NAMES = ImmutableList.of(
    "exprText",
    "className",
    "packageName",
    "cacheLevel",
    "args",
    "scopes"
  );

  public Symbol(
      String exprText,
      String className,
      String packageName,
      String cacheLevel,
      List<String> args,
      List<String> scopes) {
    super(
        FIELD_NAMES,
        ImmutableList.of(exprText, className, packageName, cacheLevel, args, scopes)
    );
  }
}
