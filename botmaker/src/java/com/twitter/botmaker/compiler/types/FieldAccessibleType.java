package com.twitter.botmaker.compiler.types;

import java.util.Set;

import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;

public interface FieldAccessibleType<T> {

  Set<String> getFieldNames();

  Type getFieldType(String fieldName) throws SemanticCheckFailure;

  boolean isSetValue(T obj, String fieldName);

  Object getFieldValue(T obj, String fieldName);
}
