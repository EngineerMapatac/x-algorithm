package com.twitter.botmaker.function.string;

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
    argTypes = {
        "String",
        "String"
    },
    arguments = {
        "the string.",
        "the suffix."
    },
    returnType = "Boolean",
    description = "Tests if a string ends with the specified suffix.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "EndsWith(\"Hello World\", \"World\")"
    }
)
public class EndsWith extends FunctionNode2<Runtime, String, String> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      Type.BOOLEAN
  );

  public EndsWith(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Boolean apply(Context<Runtime> context, String str, String suffix) {
    return str.endsWith(suffix);
  }
}
