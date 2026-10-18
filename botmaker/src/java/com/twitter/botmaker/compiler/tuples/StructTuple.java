package com.twitter.botmaker.compiler.tuples;

import java.util.List;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;

import org.apache.commons.lang3.builder.HashCodeBuilder;

public class StructTuple implements Tuple {
  private final ImmutableList<String> fieldNames;
  private final List<Object> fieldValues;

  public StructTuple(ImmutableList<String> fieldNames, List<Object> fieldValues) {
    Preconditions.checkArgument(fieldNames.size() == fieldValues.size());
    this.fieldNames = fieldNames;
    this.fieldValues = fieldValues;
  }

  @Override
  public int size() {
    return fieldValues.size();
  }

  public ImmutableList<String> getFieldNames() {
    return fieldNames;
  }

  @Override
  public Object get(int index) {
    return fieldValues.get(index);
  }

  public static StructTuple of(ImmutableList<String> fieldNames, List<Object> fieldValues) {
    return new StructTuple(fieldNames, fieldValues);
  }

  @Override
  public boolean equals(Object o) {
    if (o == this) {
      return true;
    }

    if (!(o instanceof StructTuple)) {
      return false;
    }

    StructTuple that = (StructTuple) o;
    return that.fieldNames.equals(this.fieldNames) && Tuple.equals(this, o);
  }

  @Override
  public String toString() {
    StringBuilder sb = new StringBuilder();
    sb.append('(');

    for (int i = 0; i < fieldNames.size(); i++) {
      if (i > 0) {
        sb.append(", ");
      }
      sb.append(fieldNames.get(i));
      sb.append(":");
      sb.append(fieldValues.get(i));
    }

    sb.append(')');
    return sb.toString();
  }

  @Override
  public int hashCode() {
    HashCodeBuilder hasher = new HashCodeBuilder();
    for (int i = 0; i < fieldNames.size(); i++) {
      hasher.append(fieldNames.get(i));
      hasher.append(fieldValues.get(i));
    }

    return hasher.toHashCode();
  }
}
