package com.twitter.botmaker.compiler.exceptions;


public final class MissingFeatureException extends RuntimeFailure {

  public MissingFeatureException(String msg) {
    super(msg);
  }
}
