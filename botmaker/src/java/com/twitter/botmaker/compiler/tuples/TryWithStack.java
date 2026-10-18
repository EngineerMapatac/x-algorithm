package com.twitter.botmaker.compiler.tuples;

import java.util.Arrays;
import javax.annotation.Nullable;

import com.google.common.collect.ImmutableList;

public class TryWithStack extends StructTuple {

  public static final ImmutableList<String> FIELD_NAMES = ImmutableList.of(
      "result",
      "exception",
      "message",
      "stackFrames",
      "stackTrace"
  );

  public TryWithStack(
      @Nullable Object result,
      String exceptionType,
      String exceptionMessage,
      String stackFrames,
      String stackTrace) {
    super(FIELD_NAMES,
        Arrays.asList(result, exceptionType, exceptionMessage, stackFrames, stackTrace));
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

  public String getStackFrames() {
    return (String) get(3);
  }

  public String getStackTrace() {
    return (String) get(4);
  }
}
