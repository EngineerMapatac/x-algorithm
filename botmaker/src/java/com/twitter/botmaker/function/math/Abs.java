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
    argTypes = {"Number"},
    arguments = {"input double number"},
    returnType = "Number",
    description = "Returns the absolute value of a number.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Abs(-1)"
    }
)
public class Abs extends FunctionNode1<Runtime, Number> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER),
      Type.NUMBER
  );

  public Abs(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Object apply(Context<Runtime> context, Number num) {

    if (Type.doubleLike(num)) {
      return Math.abs(num.doubleValue());
    } else if (Type.longLike(num)) {
      return Math.abs(num.longValue());
    }

    throw new IllegalArgumentException(String.format("Invalid numeric type (%s)", num));
  }
}
