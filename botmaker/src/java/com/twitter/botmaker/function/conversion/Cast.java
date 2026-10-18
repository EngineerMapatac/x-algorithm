package com.twitter.botmaker.function.conversion;

import java.nio.ByteBuffer;
import java.util.Map;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.StructTuple;
import com.twitter.botmaker.compiler.types.StructTupleType;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.function.collection.MapOperator;
import com.twitter.botmaker.function.thrift.ThriftOperator;
import com.twitter.botmaker.function.thrift.ThriftStructOperator;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;

public abstract class Cast extends FunctionNode1<Runtime, Object> {

  private Type type;
  private ASTNode node;

  private Cast(
      String exprText,
      Type type,
      ASTNode node) throws SemanticCheckFailure {
    super(exprText, ImmutableList.of(node));
    this.type = type;
    this.node = node;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public static ASTNode mkCast(
      String exprText,
      Type type,
      ASTNode node
  ) throws SemanticCheckFailure {

    try {
      final Signature signature = new Signature(
          ImmutableList.of(node.getReturnType()),
          type
      );

      if (node.getReturnType().mapLike() && type.thriftLike()) {
        ThriftType thriftType = (ThriftType) type;
        if (node instanceof MapOperator) {
          ThriftOperator.checkThriftFieldTypes(thriftType, node.getChildren());
        }

        return new Cast(exprText, type, node) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(Context<Runtime> context, Object value) throws Exception {
            if (value == null) {
              return null;
            } else {
              return thriftType.newInstance((Map<Object, Object>) value);
            }
          }
        };
      } else if (node.getReturnType().mapLike() && type.thriftStructLike()) {
        ThriftStructType thriftStructType = (ThriftStructType) type;
        if (node instanceof MapOperator) {
          ThriftStructOperator.checkThriftFieldTypes(thriftStructType, node.getChildren());
        }

        return new Cast(exprText, type, node) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(Context<Runtime> context, Object value) throws Exception {
            if (value == null) {
              return null;
            } else {
              return thriftStructType.newInstance((Map<Object, Object>) value);
            }
          }
        };
      } else if (node.getReturnType().structTupleLike() && type.structTupleLike()) {
        StructTupleType from = (StructTupleType) node.getReturnType();
        StructTupleType to = (StructTupleType) type;
        Function<StructTuple, StructTuple> converter = from.converter(to);
        return new Cast(exprText, type, node) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(Context<Runtime> context, Object value) throws Exception {
            if (value == null) {
              return null;
            }

            StructTuple structTuple = (StructTuple) value;
            return converter.apply(structTuple);
          }
        };
      } else if (node.getReturnType() == Type.BINARY && type != Type.BINARY) {
        return new Cast(exprText, type, node) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(Context<Runtime> context, Object value) throws Exception {
            if (value == null) {
              return null;
            } else {
              ByteBuffer bytes = (ByteBuffer) value;
              return type.serializer.deserialize(bytes, false);
            }
          }
        };
      } else if (node.getReturnType().numberLike() && type == Type.LONG) {
        return new ToLong(exprText, ImmutableList.of(node));
      } else if (node.getReturnType().numberLike() && type == Type.DOUBLE) {
        return new ToDouble(exprText, ImmutableList.of(node));
      } else if (!Type.isDivergentTo(node.getReturnType(), type)) {
        return new Cast(exprText, type, node) {
          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          protected Object apply(Context<Runtime> context, Object value) throws Exception {
            return value;
          }
        };
      } else {
        throw new SemanticCheckFailure(
            String.format("The return type %s of ASTNode %s cannot be cast to type %s ",
                node.getReturnType().toString(),
                node.getExprText(),
                type.toString())
        );
      }
    } catch (SemanticCheckFailure se) {
      throw se;
    } catch (Exception e) {
      throw new SemanticCheckFailure(e);
    }
  }
}
