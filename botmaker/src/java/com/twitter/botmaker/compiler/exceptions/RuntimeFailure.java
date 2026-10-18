package com.twitter.botmaker.compiler.exceptions;

public class RuntimeFailure extends RuntimeException {
  public RuntimeFailure(String msg, Throwable t) {
    super(msg, t);
  }

  public RuntimeFailure(String msg) {
    super(msg);
  }

  @Override
  public Throwable fillInStackTrace() {
    return this;
  }

}
