package com.twitter.botmaker.compiler.types;

import java.util.List;
import java.util.Map;
import java.util.Set;

import com.google.common.base.Objects;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Lists;
import com.google.common.collect.Sets;

import org.apache.commons.lang3.builder.HashCodeBuilder;

import com.twitter.botmaker.compiler.Serializer;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.StructTuple;
import com.twitter.botmaker.compiler.tuples.Tuple;
import com.twitter.util.Function;

public final class StructTupleType extends Type implements FieldAccessibleType<StructTuple> {
  public final ImmutableList<String> fieldNames;
  public final ImmutableList<Type> fieldTypes;

  private final int cachedHashCode;
  private final String cachedStringRepresentation;

  static StructTupleType ofStructTuple(
      ImmutableList<String> fieldNames, ImmutableList<Type> fieldTypes) {

    return createType(Tuple.of(StructTuple.class, fieldNames, fieldTypes), Function.exfunc0(() ->
        new StructTupleType(fieldNames, fieldTypes)
    ));

  }

  private StructTupleType(ImmutableList<String> fieldNames, ImmutableList<Type> fieldTypes) {
    super(StructTuple.class, "StructTuple",
        fieldTypes, Serializer.structTupleOf(fieldNames, fieldTypes));
    this.fieldNames = fieldNames;
    this.fieldTypes = fieldTypes;

    this.cachedHashCode = computeHashCode();
    this.cachedStringRepresentation = computeStringRepresentation();
  }

  private int computeHashCode() {
    return new HashCodeBuilder()
        .append(typeBase)
        .append(fieldNames)
        .append(fieldTypes)
        .append(NON_GENERIC_TYPE_ID)
        .toHashCode();
  }

  private String computeStringRepresentation() {
    StringBuilder builder = new StringBuilder();
    builder.append(this.typeName);
    builder.append('(');
    for (int i = 0; i < fieldNames.size(); i++) {
      if (i > 0) {
        builder.append(", ");
      }
      builder.append(fieldNames.get(i));
      builder.append(':');
      builder.append(fieldTypes.get(i));
    }

    builder.append(')');
    return builder.toString();
  }

  @Override
  public Type replaceTypeArguments(Map<Long, Type> mappedTypes) throws SemanticCheckFailure {
    ImmutableList<Type> params = this.getTypeParams();
    List<Type> inferredTypeParams = Lists.newArrayListWithCapacity(params.size());
    for (Type typeParam : params) {
      Type replacement = typeParam.replaceTypeArguments(mappedTypes);
      inferredTypeParams.add(replacement);
    }

    return ofStructTuple(fieldNames, ImmutableList.copyOf(inferredTypeParams));
  }

  public Function<StructTuple, StructTuple> converter(
      StructTupleType structTupleType) throws SemanticCheckFailure {
    int[] indexMap = new int[structTupleType.fieldNames.size()];
    for (int i = 0; i < structTupleType.fieldNames.size(); i++) {
      int fi = fieldNames.indexOf(structTupleType.fieldNames.get(i));
      if (fi < 0) {
        throw new SemanticCheckFailure(
            String.format("%s cannnot be converted to %s. missing field %s.",
                this,
                structTupleType,
                structTupleType.fieldNames.get(i))
        );
      }
      if (!fieldTypes.get(fi).equals(structTupleType.fieldTypes.get(i))) {
        throw new SemanticCheckFailure(
            String.format("%s cannnot be converted to %s. field type %s mismatch.",
                this,
                structTupleType,
                structTupleType.fieldNames.get(i))
        );
      }
      indexMap[i] = fi;
    }

    return Function.func((StructTuple t) -> {
      List<Object> fieldValues = Lists.newArrayListWithCapacity(indexMap.length);
      for (int i = 0; i < indexMap.length; i++) {
        fieldValues.add(t.get(indexMap[i]));
      }
      return StructTuple.of(
          structTupleType.fieldNames,
          fieldValues
      );
    });

  }

  @Override
  public Set<String> getFieldNames() {
    Set<String> fNames = Sets.newHashSetWithExpectedSize(fieldNames.size() * 2);
    fNames.addAll(fieldNames);
    for (int i = 0; i < fieldNames.size(); ++i) {
      fNames.add("_" + (i + 1));
    }
    return ImmutableSet.copyOf(fNames);
  }

  @Override
  public Type getFieldType(String fieldName) throws SemanticCheckFailure {
    int fieldIndex = fieldNames.indexOf(fieldName);
    if (fieldIndex != -1) {
      return fieldTypes.get(fieldIndex);
    } else {
      if (!fieldName.startsWith("_")) {
        throw new SemanticCheckFailure(
            String.format(
                "field name %s is not found in struct tuple class %s",
                fieldName,
                toString()));
      }

      try {
        return typeParams.get(Integer.valueOf(fieldName.substring(1)) - 1);
      } catch (Throwable t) {
        throw new SemanticCheckFailure(t);
      }

    }
  }

  @Override
  public boolean isSetValue(StructTuple obj, String fieldName) {
    if (fieldNames.contains(fieldName)) {
      return true;
    } else {
      try {
        return (fieldName.startsWith("_"))
            && Integer.valueOf(fieldName.substring(1)) - 1 < fieldNames.size();
      } catch (Throwable t) {
        return false;
      }
    }
  }

  @SuppressWarnings("unchecked")
  @Override
  public Object getFieldValue(StructTuple obj, String fieldName) {
    int fieldIndex = fieldNames.indexOf(fieldName);
    if (fieldIndex == -1) {
      fieldIndex = Integer.valueOf(fieldName.substring(1)) - 1;
    }
    return obj.get(fieldIndex);
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    StructTupleType that = (StructTupleType) o;
    return Objects.equal(fieldNames, that.fieldNames)
        && Objects.equal(fieldTypes, that.fieldTypes);
  }

  @Override
  public int hashCode() {
    return cachedHashCode;
  }

  @Override
  public String toString() {
    return cachedStringRepresentation;
  }
}
