package com.twitter.botmaker.function.collection;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.tuples.StructTuple;
import com.twitter.botmaker.compiler.types.StructTupleType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.Constant;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.function.conversion.ToPair;

public abstract class Copy extends FunctionNode<Runtime> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();

  public Copy(String exprText, List<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public static Copy of(ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    if (children.size() < 1) {
      throw new SemanticCheckFailure("expects at least one argument.");
    }

    Type nodeType = children.get(0).getReturnType();
    if (nodeType.mapLike()) {
      return ofMap(children);
    } else if (nodeType.structTupleLike()) {
      return ofStructTuple(children);
    } else {
      throw new SemanticCheckFailure("Copy function only supports Map or StructTuple.");
    }

  }

  private static Copy ofMap(ImmutableList<ASTNode> children) throws SemanticCheckFailure {

    for (int i = 1; i < children.size(); i++) {
      if (!children.get(i).getReturnType().pairLike()) {
        throw new SemanticCheckFailure(String.format(
            "expects a Pair argument but get %s.", children.get(i).getReturnType())
        );
      }
    }

    Signature signature = new Signature(
        ImmutableList.of(Type.mapOf(TPA, TPB)),
        Type.pairOf(TPA, TPB),
        Type.mapOf(TPA, TPB)
    );

    return new Copy("Copy", children) {

      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(Context<Runtime> context, List<Object> args) {
        if (args.size() == 1) {
          return args.get(0);
        }

        HashMap<Object, Object> result = Maps.newHashMap();
        Map<Object, Object> map = (Map<Object, Object>) args.get(0);
        for (Map.Entry entry : map.entrySet()) {
          result.put(entry.getKey(), entry.getValue());
        }

        for (int i = 1; i < args.size(); i++) {
          Pair pair = (Pair) args.get(i);
          result.put(pair.getFirst(), pair.getSecond());
        }

        return ImmutableMap.copyOf(result);

      }
    };
  }

  private static Copy ofStructTuple(ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    StructTupleType structTupleType = (StructTupleType) children.get(0).getReturnType();

    List<String> fieldNames = Lists.newArrayList(structTupleType.fieldNames);
    List<Type> fieldTypes = Lists.newArrayList(structTupleType.fieldTypes);

    ImmutableList.Builder<Type> argTypeBuilder = ImmutableList.builder();
    argTypeBuilder.add(structTupleType);

    ImmutableList.Builder<ASTNode> argBuilder = ImmutableList.builder();
    argBuilder.add(children.get(0));

    int[] overriddenFieldIndices = new int[children.size()];

    for (int i = 1; i < children.size(); i++) {
      ASTNode child = children.get(i);

      if (!(child instanceof ToPair)) {
        throw new SemanticCheckFailure(
            String.format("expects <name>: <value> pairs, but a %s received", child));
      }

      String fieldName = ((Constant) child.getChildren().get(0)).getValue().toString();
      Type fieldType = ((ASTNode) child.getChildren().get(1)).getReturnType();

      argBuilder.add((ASTNode) child.getChildren().get(1));
      argTypeBuilder.add(fieldType);

      int fieldIndex = fieldNames.indexOf(fieldName);
      if (fieldIndex < 0) {
        fieldNames.add(fieldName);
        fieldTypes.add(fieldType);
        overriddenFieldIndices[i] = fieldNames.size() - 1;
      } else if (!Type.isDivergentTo(fieldTypes.get(fieldIndex), fieldType)) {
        overriddenFieldIndices[i] = fieldIndex;
      } else {
        throw new SemanticCheckFailure(
            String.format("StructTuple field %s expects type %s, but %s received.",
                fieldName, fieldTypes.get(fieldIndex), fieldType)
        );
      }
    }

    StructTupleType returnType = Type.structTupleOf(
        ImmutableList.copyOf(fieldNames),
        ImmutableList.copyOf(fieldTypes)
    );

    Signature signature = new Signature(
        argTypeBuilder.build(),
        returnType
    );

    return new Copy("Copy", argBuilder.build()) {

      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(Context<Runtime> context, List<Object> args) {
        if (args.size() == 1) {
          return args.get(0);
        }

        StructTuple structTuple = (StructTuple) args.get(0);

        Object[] fieldValues = new Object[returnType.fieldNames.size()];
        for (int i = 0; i < structTuple.size(); i++) {
          fieldValues[i] = structTuple.get(i);
        }

        for (int i = 1; i < args.size(); i++) {
          fieldValues[overriddenFieldIndices[i]] = args.get(i);
        }

        return StructTuple.of(returnType.fieldNames, Arrays.asList(fieldValues));
      }
    };
  }
}
