package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.types.Type;
import com.twitter.scrooge.ThriftEnum;

public class ThriftEnumMetaData extends FieldValueMetaData {
  public final Type type;
  public final Class<? extends ThriftEnum> enumClass;

  public ThriftEnumMetaData(Class<? extends ThriftEnum> sClass) {
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
