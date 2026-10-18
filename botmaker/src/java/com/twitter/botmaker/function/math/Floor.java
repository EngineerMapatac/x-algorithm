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
    arguments = {"number num"},
    returnType = "Double",
    description = "Rounds a number down to the nearest long.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Floor(-1.8)"
    }
)
public class Floor extends FunctionNode1<Runtime, Number> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER),
      Type.DOUBLE
  );

  public Floor(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
    return Math.floor(num.doubleValue());
  }
}
