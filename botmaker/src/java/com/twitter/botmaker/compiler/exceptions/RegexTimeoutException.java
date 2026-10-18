package com.twitter.botmaker.compiler.exceptions;



public final class RegexTimeoutException extends RuntimeFailure {

  public RegexTimeoutException(String featureName) {
    super(featureName);
  }
}
