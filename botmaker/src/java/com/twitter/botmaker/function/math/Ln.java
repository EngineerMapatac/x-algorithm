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
    returnType = "Double",
    description = "Returns the Natural Logarithm of a double number, return -Infinity when"
        + " input is zero, NaN when input is negative",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Ln(1)"
    }
)
public class Ln extends FunctionNode1<Runtime, Number> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER),
      Type.DOUBLE
  );

  public Ln(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected Object apply(Context<Runtime> context, Number num) {
    return Math.log(num.doubleValue());
  }
}
