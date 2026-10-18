package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.scrooge.ThriftStruct;

public class ThriftStructMetaData extends FieldValueMetaData {
  public final ThriftStructType type;
  public final Class<? extends ThriftStruct> structClass;

  public ThriftStructMetaData(Class<? extends ThriftStruct> sClass) throws SemanticCheckFailure {
    super(TType.STRUCT);
    this.structClass = sClass;
    this.type = Type.thriftStructOf(structClass);
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
