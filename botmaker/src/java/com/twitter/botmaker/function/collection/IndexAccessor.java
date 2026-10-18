package com.twitter.botmaker.function.collection;

import java.util.List;
import java.util.Map;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

public abstract class IndexAccessor extends FunctionNode2<Runtime, Object, Object> {

  private IndexAccessor(
      String exprText,
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public static ASTNode mkIndexAccessor(
      String exprText, ASTNode exprNode, ASTNode indexNode) throws SemanticCheckFailure {

    Type returnType = exprNode.getReturnType();
    Type indextype = indexNode.getReturnType();
    if (returnType.getBase().equals(List.class)
        && returnType.getTypeParams().size() == 1
        && indextype.equals(Type.pairOf(Type.LONG, Type.LONG))) {
      return listPairAccessor(exprText, exprNode, indexNode);
    } else if (returnType.getBase().equals(List.class)
        && returnType.getTypeParams().size() == 1) {
      return listIndexAccessor(exprText, exprNode, indexNode);
    } else if (returnType.getBase().equals(Map.class)
        && returnType.getTypeParams().size() == 2) {
      return mapKeyAccessor(exprText, exprNode, indexNode);
    } else if (returnType.getBase().equals(Object.class)) {
      return objectIndexAccessor(exprText, exprNode, indexNode);
    } else {
      throw new SemanticCheckFailure(
          String.format("Type %s does not support index accessor", returnType));
    }
  }

  private static ASTNode listPairAccessor(
      String exprText, ASTNode exprNode, ASTNode indexNode) throws SemanticCheckFailure {

    Type returnType = exprNode.getReturnType();
    Type indextype = indexNode.getReturnType();

    Signature signature = new Signature(
        ImmutableList.of(returnType, indextype),
        returnType);
    return new IndexAccessor(exprText, ImmutableList.of(exprNode, indexNode)) {
      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(Context<Runtime> context, Object arg1, Object arg2) {
        List list = (List) arg1;
        Pair<Long, Long> index = (Pair<Long, Long>) arg2;
        return list.subList(
            index.getFirst().intValue(),
            Math.min(list.size(), index.getSecond().intValue())
        );
      }
    };
  }

  private static ASTNode listIndexAccessor(
      String exprText, ASTNode exprNode, ASTNode indexNode) throws SemanticCheckFailure {

    Type returnType = exprNode.getReturnType();
    Type indextype = indexNode.getReturnType();
    Signature signature = new Signature(
        ImmutableList.of(returnType, Type.LONG),
        returnType.getTypeParams().get(0));

    return new IndexAccessor(exprText, ImmutableList.of(exprNode, indexNode)) {
      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(Context<Runtime> context, Object arg1, Object arg2) {
        List list = (List) arg1;
        long index = (Long) arg2;
        return list.get((int) index);
      }
    };
  }

  private static ASTNode mapKeyAccessor(
      String exprText, ASTNode exprNode, ASTNode indexNode) throws SemanticCheckFailure {

    Type returnType = exprNode.getReturnType();
    Type indextype = indexNode.getReturnType();

    Signature signature = new Signature(
        ImmutableList.of(returnType, returnType.getTypeParams().get(0)),
        returnType.getTypeParams().get(1));
    return new IndexAccessor(exprText, ImmutableList.of(exprNode, indexNode)) {
      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(
          Context<Runtime> context,
          Object arg1,
          Object arg2
      ) {
        Map map = (Map) arg1;
        Object key = arg2;
        return map.get(key);
      }
    };
  }

  private static ASTNode objectIndexAccessor(
      String exprText, ASTNode exprNode, ASTNode indexNode) throws SemanticCheckFailure {

    Type returnType = exprNode.getReturnType();
    Type indextype = indexNode.getReturnType();

    Signature signature = new Signature(
        ImmutableList.of(Type.OBJECT, Type.LONG),
        Type.OBJECT);
    return new IndexAccessor(exprText, ImmutableList.of(exprNode, indexNode)) {
      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(
          Context<Runtime> context,
          Object arg1,
          Object arg2
      ) {
        List list = (List) arg1;
        long index = (Long) arg2;
        return list.get((int) index);
      }
    };

  }

}
