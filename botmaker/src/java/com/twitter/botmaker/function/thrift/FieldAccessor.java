package com.twitter.botmaker.function.thrift;

import java.util.List;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Tuple;
import com.twitter.botmaker.compiler.tuples.Tuple3;
import com.twitter.botmaker.compiler.types.FieldAccessibleType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;

public abstract class FieldAccessor extends FunctionNode1<Runtime, Object> {

  private FieldAccessor(
      String exprText,
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  private static ASTNode.Signature getSignature(ASTNode node, Type valueType) {
    return new ASTNode.Signature(ImmutableList.of(node.getReturnType()), valueType);
  }

  @SuppressWarnings("unchecked")
  public static ASTNode mkFieldAccessor(
      String exprText,
      ASTNode node,
      ImmutableList<String> fields
  ) throws SemanticCheckFailure {



    if (!(node.getReturnType() instanceof FieldAccessibleType)) {
      throw new SemanticCheckFailure(
          String.format("expect a FieldAccessibleType, but %s received",
              node.getReturnType().toString())
      );
    }

    List<Tuple3<FieldAccessibleType, Type, Function<Object, Object>>> fieldInfosBuilder =
        Lists.newArrayListWithCapacity(fields.size());
    FieldAccessibleType objType = (FieldAccessibleType) node.getReturnType();
    for (String field : fields.subList(0, fields.size() - 1)) {
      Tuple3<FieldAccessibleType, Type, Function<Object, Object>> fieldInfo =
          getFieldInfo(objType, field);
      fieldInfosBuilder.add(fieldInfo);
      if (!(fieldInfo.getSecond() instanceof FieldAccessibleType)) {
        throw new SemanticCheckFailure(
            String.format("expect a FieldAccessibleType, but %s received",
                fieldInfo.getSecond().toString())
        );
      }
      objType = (FieldAccessibleType) fieldInfo.getSecond();
    }

    Tuple3<FieldAccessibleType, Type, Function<Object, Object>> lastFieldInfo =
        getFieldInfo(objType, fields.get(fields.size() - 1));
    fieldInfosBuilder.add(lastFieldInfo);
    Type valueType = lastFieldInfo.getSecond();

    ASTNode.Signature signature = getSignature(node, valueType);
    ImmutableList<Tuple3<FieldAccessibleType, Type, Function<Object, Object>>> fieldInfos =
        ImmutableList.copyOf(fieldInfosBuilder);

    return new FieldAccessor(exprText, ImmutableList.of(node)) {
      @Override
      protected Object apply(Context<Runtime> context, Object arg) {
        Object value = arg;
        for (Tuple3<FieldAccessibleType, Type, Function<Object, Object>> info : fieldInfos) {
          if (value == null) {
            return null;
          }
          value = info.getThird().apply(value);
        }
        return value;
      }

      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected void computeFingerprint(Hasher hasher) {
        super.computeFingerprint(hasher);

        hasher.putLong(fields.size());
        for (String field : fields) {
          hasher.putUnencodedChars(field);
        }
      }
    };
  }

  private static Tuple3<FieldAccessibleType, Type, Function<Object, Object>> getFieldInfo(
      FieldAccessibleType objType, String fieldName) throws SemanticCheckFailure {
    Type fieldType = objType.getFieldType(fieldName);
    return Tuple.of(
        objType,
        fieldType,
        Function.func(obj -> {
          if (obj != null && objType.isSetValue(obj, fieldName)) {
            return objType.getFieldValue(obj, fieldName);
          } else {
            return null;
          }
        })
    );
  }
}
