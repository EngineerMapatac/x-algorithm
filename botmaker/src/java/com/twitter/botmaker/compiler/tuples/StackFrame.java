package com.twitter.botmaker.compiler.tuples;

import java.util.Arrays;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.compiler.types.Type;

public class StackFrame extends StructTuple {

  public static final ImmutableList<String> FIELD_NAMES = ImmutableList.of(
      "expr",
      "lineInParentFrame"
  );

  public static final Type TYPE = Type.structTupleOf(
      FIELD_NAMES,
      ImmutableList.of(Type.STRING, Type.LONG)
  );

  public StackFrame(String expr, long line) {
    super(FIELD_NAMES, Arrays.asList(expr, line));
  }

  public String getExpr() {
    return (String) get(0);
  }

  public Long getLineInParentFrame() {
    return (Long) get(1);
  }
}
