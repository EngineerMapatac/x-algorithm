package com.twitter.botmaker.function.math;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode0;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {},
    arguments = {},
    returnType = "Double",
    description = "Random number greater than or equal to 0.0 and less than 1.0",
    actionLevel = ActionLevel.NO_ACTION
)
public class Random extends FunctionNode0<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.<Type>of(),
      Type.DOUBLE
  );

  public Random(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Never;
  }

  @Override
  public Double apply(Context<Runtime> context) {
    return Math.random();
  }
}
