package com.twitter.botmaker.function.string;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.RegexCache;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode3;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "String",
        "String",
        "String"
    },
    arguments = {
        "original string",
        "regex",
        "replacement"
    },
    returnType = "String",
    description = "Replace the target regex with replacement string in the given string.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Replace(\"Abcd efg\", \" \", \",\")"
    }
)
public class Replace extends FunctionNode3<Runtime, String, String, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(
          Type.STRING,
          Type.STRING,
          Type.STRING),
      Type.STRING
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public Replace(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected String apply(
      Context<Runtime> context, String str, String target, String replacement) {

    final RegexCache regexCache;
    if (context.getRuntime() instanceof RegexCache) {
      regexCache = (RegexCache) (context.getRuntime());
    } else {
      regexCache = RegexCache.DEFAULT;
    }

    return regexCache.replace(context, str, target, replacement);
  }
}
