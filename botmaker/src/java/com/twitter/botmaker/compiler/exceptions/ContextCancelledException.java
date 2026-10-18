package com.twitter.botmaker.compiler.exceptions;


public final class ContextCancelledException extends RuntimeFailure {

  public ContextCancelledException() {
    super("context was cancelled.");
  }
}
