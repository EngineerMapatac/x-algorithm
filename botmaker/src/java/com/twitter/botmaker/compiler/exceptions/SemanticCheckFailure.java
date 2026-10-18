package com.twitter.botmaker.compiler.exceptions;

public class SemanticCheckFailure extends Exception {

  public SemanticCheckFailure(String msg) {
    super(msg);
  }

  public SemanticCheckFailure(Throwable t) {
    super(t);
  }
}
