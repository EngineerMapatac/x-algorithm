package com.twitter.botmaker.compiler.thrift;

import org.apache.thrift.TBase;

import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;

public class TBaseStructMetaData extends FieldValueMetaData {
  public final ThriftType type;
  public final Class<? extends TBase> structClass;

  public TBaseStructMetaData(Class<? extends TBase> sClass) throws SemanticCheckFailure {
    super(TType.STRUCT);
    this.structClass = sClass;
    this.type = Type.thriftOf(structClass);
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
