package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.types.Type;

public class SetMetaData extends FieldValueMetaData {
  public final Type type;
  public final Type rawType;

  public final FieldValueMetaData elemMetaData;

  public SetMetaData(FieldValueMetaData eMetaData) {
    super(TType.SET);
    this.elemMetaData = eMetaData;
    this.type = Type.setOf(eMetaData.getType());
    this.rawType = Type.setOf(eMetaData.getRawType());
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

