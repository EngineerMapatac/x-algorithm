package com.twitter.botmaker.function.string;

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
    argTypes = {"Long"},
    arguments = {"Unicode of the character"},
    returnType = "String",
    description = "A single character string of the given unicode value.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {"Chr(32)"}
)
public class Chr extends FunctionNode1<Runtime, Long> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG),
      Type.STRING
  );

  public Chr(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected String apply(Context<Runtime> context, Long value) {
    char unicode = (char) (long) value;
    return String.valueOf(unicode);
  }
}
