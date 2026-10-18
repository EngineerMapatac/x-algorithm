package com.twitter.botmaker.compiler.exceptions;

public final class ParseFailure extends Exception {
  public ParseFailure(String msg, Throwable t) {
    super(msg, t);
  }

  public ParseFailure(String msg) {
    super(msg);
  }
}
