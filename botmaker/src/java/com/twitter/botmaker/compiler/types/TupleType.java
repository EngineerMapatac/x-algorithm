package com.twitter.botmaker.compiler.types;

import java.util.Set;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Sets;

import com.twitter.botmaker.compiler.Serializer;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.tuples.Tuple;
import com.twitter.util.Function;

public final class TupleType extends Type implements FieldAccessibleType<Tuple> {

  private TupleType(
      Class<?> typeBase,
      String typeName,
      ImmutableList<Type> typeParams,
      long genericTypeId,
      Serializer serializer) {
    super(typeBase, typeName, typeParams, genericTypeId, serializer);
  }

  public static TupleType ofTuple(
      Class<?> typeBase,
      ImmutableList<Type> typeParams,
      Serializer serializer) {
    Preconditions.checkArgument(isClassSupported(typeBase));
    return createType(
        Pair.of(typeBase, typeParams),
        Function.func0(() ->
            new TupleType(
                typeBase,
                typeBase.getSimpleName(),
                typeParams,
                NON_GENERIC_TYPE_ID,
                serializer)
        ));
  }

  @Override
  public Set<String> getFieldNames() {
    Set<String> fieldNames = Sets.newHashSetWithExpectedSize(typeParams.size());
    for (int i = 0; i < typeParams.size(); ++i) {
      fieldNames.add("_" + (i + 1));
    }
    return ImmutableSet.copyOf(fieldNames);
  }

  @Override
  public Type getFieldType(String fieldName) throws SemanticCheckFailure {
    if (!fieldName.startsWith("_")) {
      throw new SemanticCheckFailure(
          String.format(
              "field name %s is not an underscore followed by a number for tuple class %s",
              fieldName,
              toString()));
    }

    try {
      return typeParams.get(Integer.valueOf(fieldName.substring(1)) - 1);
    } catch (Throwable t) {
      throw new SemanticCheckFailure(t);
    }
  }

  @Override
  public boolean isSetValue(Tuple obj, String fieldName) {
    try {
      return (fieldName.startsWith("_"))
          && Integer.valueOf(fieldName.substring(1)) - 1 < typeParams.size();
    } catch (Throwable t) {
      return false;
    }
  }

  @SuppressWarnings("unchecked")
  @Override
  public Object getFieldValue(Tuple obj, String fieldName) {
    return obj.get(Integer.valueOf(fieldName.substring(1)) - 1);
  }
}
