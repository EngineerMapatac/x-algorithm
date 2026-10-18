package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.types.Type;

public class ListMetaData extends FieldValueMetaData {
  public final Type type;
  public final Type rawType;

  public final FieldValueMetaData elemMetaData;

  public ListMetaData(FieldValueMetaData eMetaData) {
    super(TType.LIST);
    this.elemMetaData = eMetaData;
    this.type = Type.listOf(eMetaData.getType());
    this.rawType = Type.listOf(eMetaData.getRawType());
  }

  @Override
  public Type getType() {
    return type;
  }

  @Override
  public Type getRawType() {
    return rawType;
  }
}

