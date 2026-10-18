package com.twitter.botmaker.compiler;

import com.twitter.botmaker.compiler.exceptions.RegexTimeoutException;

public class InterruptibleCharSequence implements CharSequence {
  private final CharSequence underlying;
  private final Timeout timeout;
  private long checker = 0L;

  public InterruptibleCharSequence(CharSequence underlying, Timeout timeout) {
    super();
    this.underlying = underlying;
    this.timeout = timeout;
  }

  @Override
  public int length() {
    return underlying.length();
  }

  @Override
  public char charAt(int index) {

    if ((++checker & 0x3FL) == 0) {
      if (timeout.isDone()) {
        throw new RegexTimeoutException(
            String.format(
                "regex match being interrupted. Tmeout: %s.", timeout
            ));
      }
    }
    return underlying.charAt(index);
  }

  @Override
  public CharSequence subSequence(int start, int end) {
    if (underlying instanceof InterruptibleCharSequence) {
      return underlying.subSequence(start, end);
    } else {
      return new InterruptibleCharSequence(underlying.subSequence(start, end), timeout);
    }
  }

  @Override
  public String toString() {
    return underlying.toString();
  }
}
