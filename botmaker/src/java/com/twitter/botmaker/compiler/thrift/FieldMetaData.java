package com.twitter.botmaker.compiler.thrift;

import java.io.Serializable;

public class FieldMetaData implements Serializable {
  public final String fieldName;
  public final short fieldId;
  public final byte requirementType;
  public final FieldValueMetaData valueMetaData;

  public FieldMetaData(String name, short id, byte req, FieldValueMetaData vMetaData) {
    this.fieldName = name;
    this.fieldId = id;
    this.requirementType = req;
    this.valueMetaData = vMetaData;
  }
}

