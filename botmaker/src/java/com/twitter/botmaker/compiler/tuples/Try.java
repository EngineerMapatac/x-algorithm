package com.twitter.botmaker.compiler.tuples;

import java.util.Arrays;
import javax.annotation.Nullable;

import com.google.common.collect.ImmutableList;

public class Try extends StructTuple {

  public static final ImmutableList<String> FIELD_NAMES = ImmutableList.of(
      "result",
      "exception",
      "message"
  );

  public Try(
      @Nullable Object result,
      String exceptionType,
      String exceptionMessage) {
    super(FIELD_NAMES, Arrays.asList(result, exceptionType, exceptionMessage));
  }

  public Object getResult() {
    return get(0);
  }

  public String getExceptionType() {
    return (String) get(1);
  }

  public String getExceptionMessage() {
    return (String) get(2);
  }
}
