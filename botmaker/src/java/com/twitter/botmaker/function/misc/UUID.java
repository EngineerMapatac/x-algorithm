package com.twitter.botmaker.function.misc;

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
    returnType = "String",
    description = "Returns a randomly generated UUID as string",
    actionLevel = ActionLevel.NO_ACTION
)

public class UUID extends FunctionNode0<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.STRING
  );

  public UUID(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected Object apply(Context<Runtime> context) {
    return java.util.UUID.randomUUID().toString();
  }
}
