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
    arguments = {"the string to be quoted"},
    returnType = "String",
    description = "adding quotes to the string",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Quotes(\"hi\")"
    }
)
public class Quotes extends FunctionNode1<Runtime, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.<Type>of(
          Type.STRING),
      Type.STRING
  );

  public Quotes(String exprText, ImmutableList<ASTNode> children)
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
  public Object apply(Context<Runtime> context, String str) {
    return str == null ? "\"\"" : "\"" + str + "\"";
  }
}
