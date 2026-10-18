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
    argTypes = {"String"},
    arguments = {"string value to trim"},
    returnType = "String",
    description =
        "Returns a string whose value is this string,"
            + " with any leading and trailing whitespace removed",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Trim(\"  test   \")"
    }
)
public class Trim extends FunctionNode1<Runtime, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      Type.STRING
  );

  public Trim(String exprText, ImmutableList<ASTNode> children)
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
  public Object apply(Context<Runtime> context, String arg) {
    return arg.trim();
  }
}
