package com.twitter.botmaker.compiler.types;

import java.util.Objects;

public final class Symbol {
  private final String value;

  public Symbol(String value) {
    this.value = value.intern();
  }

  public String getValue() {
    return this.value;
  }

  @Override
  public boolean equals(Object other) {
    return other instanceof Symbol && this.value.equals(((Symbol) other).value);
  }

  @Override
  public int hashCode() {
    return Objects.hash(this.getClass().getName(), this.value);
  }

  public String debugString() {
    return String.format("`%s`", this.value);
  }

  @Override
  public String toString() {
    return this.value;
  }
}
