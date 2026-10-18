package com.twitter.botmaker.function.string;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2O1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "String",
        "String",
        "[Long]"
    },
    arguments = {
        "the string.",
        "the prefix.",
        "where to begin looking in the string (default = 0)."
    },
    returnType = "Boolean",
    description = "Tests if the substring of a string beginning at the specified index starts"
        + " with the specified prefix.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "StartsWith(\"Hello World\", \"Hello\")"
    }
)
public class StartsWith extends FunctionNode2O1<Runtime, String, String, Long> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      ImmutableList.of(Type.LONG),
      Type.BOOLEAN
  );

  public StartsWith(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
      Context<Runtime> context, String str, String prefix, Long offset) {
    return str.startsWith(prefix, offset.intValue());
  }

  @Override
  protected Object apply(Context<Runtime> context, String str, String prefix) {
    return str.startsWith(prefix, 0);
  }
}
