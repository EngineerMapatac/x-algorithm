package com.twitter.botmaker.function.logic;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    internal = true,
    argTypes = {},
    arguments = {},
    returnType = "Boolean",
    description = "Internal use only.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "( LEFT < RIGHT )"
    }
)
public class LessThan extends FunctionNode2<Runtime, Number, Number> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER, Type.NUMBER),
      Type.BOOLEAN
  );

  public LessThan(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Number argA, Number argB) {
    if (Type.longLike(argA) && Type.longLike(argB)) {
      return argA.longValue() < argB.longValue();
    } else {
      return argA.doubleValue() < argB.doubleValue();
    }
  }
}
