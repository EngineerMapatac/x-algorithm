package com.twitter.botmaker.function.collection;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Tuple;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

public abstract class ToTuple extends FunctionNode<Runtime> {

  public ToTuple(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  protected Object apply(Context<Runtime> context, List<Object> args) throws Exception {
    return Tuple.of(args.toArray());
  }

  public static ToTuple of(List<ASTNode> children) throws SemanticCheckFailure {

    ImmutableList.Builder<Type> builder = ImmutableList.builder();
    for (ASTNode child : children) {
      builder.add(child.getReturnType());
    }

    ImmutableList<Type> typeParams = builder.build();
    Signature signature = new Signature(
        typeParams,
        Type.tupleOf(typeParams.toArray(new Type[typeParams.size()]))
    );

    return new ToTuple("Tuple", ImmutableList.copyOf(children)) {

      @Override
      public Signature getSignature() {
        return signature;
      }
    };
  }
}
