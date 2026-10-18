package com.twitter.botmaker.function.conversion;

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
    argTypes = {"Object"},
    arguments = {"string to convert to a long"},
    returnType = "Long",
    description = "Long representation of the string.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ToLong(\"1234567\")"
    }
)
public class ToLong extends FunctionNode1<Runtime, Object> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT),
      Type.LONG
  );

  public ToLong(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Object arg) {

    if (arg instanceof Number) {
      return ((Number) arg).longValue();
    }

    return Long.valueOf(arg.toString());
  }
}
