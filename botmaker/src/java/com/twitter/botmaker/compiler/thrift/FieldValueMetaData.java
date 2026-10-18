package com.twitter.botmaker.compiler.thrift;

import java.io.Serializable;

import com.twitter.botmaker.compiler.types.Type;

public abstract class FieldValueMetaData implements Serializable {
  public final byte ttype;
  private final boolean isTypedefType;
  private final String typedefName;

  public FieldValueMetaData(byte ttype) {
    this.ttype = ttype;
    this.isTypedefType = false;
    this.typedefName = null;
  }

  public FieldValueMetaData(byte ttype, String typedefName) {
    this.ttype = ttype;
    this.isTypedefType = true;
    this.typedefName = typedefName;
  }

  public boolean isTypedef() {
    return this.isTypedefType;
  }

  public String getTypedefName() {
    return this.typedefName;
  }

  public boolean isStruct() {
    return this.ttype == 12;
  }

  public boolean isBinary() {
    return this.ttype == TType.BINARY;
  }

  public boolean isContainer() {
    return this.ttype == 15 || this.ttype == 13 || this.ttype == 14;
  }

  public byte getTType() {
    return isBinary() ? TType.STRING : ttype;
  }

  public abstract Type getType();

  public abstract Type getRawType();
}
