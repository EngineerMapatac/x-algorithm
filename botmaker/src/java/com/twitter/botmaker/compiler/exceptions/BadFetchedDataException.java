package com.twitter.botmaker.compiler.exceptions;



public final class BadFetchedDataException extends RuntimeFailure {

  public BadFetchedDataException(String featureName) {
    super(featureName);
  }
}
