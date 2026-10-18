package com.twitter.botmaker.compiler.thrift;

import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;

public class PrimitiveValueMetaData extends FieldValueMetaData {
  public final Type type;
  public final Type rawType;

  public PrimitiveValueMetaData(byte ttype) throws SemanticCheckFailure {
    super(ttype);

    switch (ttype) {
      case TType.STRING:
        type = Type.STRING;
        rawType = Type.STRING;
        break;

      case TType.BOOL:
        type = Type.BOOLEAN;
        rawType = Type.BOOLEAN;
        break;

      case TType.BINARY:
        type = Type.BINARY;
        rawType = Type.BINARY;
        break;

      case TType.BYTE:
        type = Type.LONG;
        rawType = Type.BYTE;
        break;

      case TType.I16:
        type = Type.LONG;
        rawType = Type.SHORT;
        break;

      case TType.I32:
        type = Type.LONG;
        rawType = Type.INTEGER;
        break;

      case TType.I64:
        type = Type.LONG;
        rawType = Type.LONG;
        break;

      case TType.DOUBLE:
        type = Type.DOUBLE;
        rawType = Type.DOUBLE;
        break;

      default:
        throw new SemanticCheckFailure(
            String.format("Unsupported field type %s", ttype));
    }
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

