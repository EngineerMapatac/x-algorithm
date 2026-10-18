package com.twitter.botmaker.compiler.thrift;

import org.apache.thrift.TEnum;

import com.twitter.botmaker.compiler.types.Type;

public class TEnumMetaData extends FieldValueMetaData {
  public final Type type;
  public final Class<? extends TEnum> enumClass;

  public TEnumMetaData(Class<? extends TEnum> sClass) {
    super(TType.ENUM);
    this.enumClass = sClass;
    this.type = Type.STRING;
  }

  @Override
  public Type getType() {
    return type;
  }

  @Override
  public Type getRawType() {
    return type;
  }
}
