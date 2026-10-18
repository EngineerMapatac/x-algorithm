package com.twitter.botmaker.function.datetime;

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
    returnType = "Long",
    description = "The current time in milliseconds",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {"CurrentTimeMs()"}
)
public class CurrentTimeMs extends FunctionNode0<Runtime> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.LONG
  );

  public CurrentTimeMs(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context) throws Exception {
    return context.nowMillis();
  }
}
