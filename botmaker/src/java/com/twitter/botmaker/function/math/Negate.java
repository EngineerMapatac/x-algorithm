package com.twitter.botmaker.function.math;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    internal = true,
    argTypes = {},
    arguments = {},
    returnType = "Boolean",
    description = "Internal use only.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "! ( LEFT == RIGHT )"
    }
)
public class Negate extends FunctionNode1<Runtime, Boolean> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.BOOLEAN),
      Type.BOOLEAN
  );

  public Negate(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  protected Object apply(
      Context<Runtime> context,
      Boolean value
  ) throws Exception {
    return !value;
  }

}
