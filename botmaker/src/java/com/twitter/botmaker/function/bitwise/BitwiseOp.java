package com.twitter.botmaker.function.bitwise;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

public abstract class BitwiseOp extends FunctionNode2<Runtime, Long, Long> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG, Type.LONG),
      Type.LONG
  );

  public BitwiseOp(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, Long a, Long b) {
    return doTheMath(a, b);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  abstract Long doTheMath(Long a, Long b);
}
