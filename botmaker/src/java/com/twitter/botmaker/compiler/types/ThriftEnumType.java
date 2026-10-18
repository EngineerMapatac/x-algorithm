package com.twitter.botmaker.compiler.types;

import org.apache.thrift.TEnum;

import com.twitter.botmaker.compiler.Serializer;
import com.twitter.util.Function;

public final class ThriftEnumType extends Type {

  private ThriftEnumType(Class<? extends TEnum> thriftClass) {
    super(thriftClass, thriftClass.getName(), Serializer.thriftEnumOf(thriftClass));
  }

  static ThriftEnumType ofThriftEnum(Class<? extends TEnum> typeEnum) {

    return createType(typeEnum, Function.exfunc0(() ->
        new ThriftEnumType(typeEnum)
    ));

  }
}
