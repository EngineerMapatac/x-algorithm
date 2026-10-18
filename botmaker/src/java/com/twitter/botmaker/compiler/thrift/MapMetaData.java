package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.types.Type;

public class MapMetaData extends FieldValueMetaData {
  public final Type type;
  public final Type rawType;

  public final FieldValueMetaData keyMetaData;
  public final FieldValueMetaData valueMetaData;

  public MapMetaData(FieldValueMetaData kMetaData, FieldValueMetaData vMetaData) {
    super(TType.MAP);
    this.keyMetaData = kMetaData;
    this.valueMetaData = vMetaData;

    this.type = Type.mapOf(kMetaData.getType(), vMetaData.getType());
    this.rawType = Type.mapOf(kMetaData.getRawType(), vMetaData.getRawType());
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

