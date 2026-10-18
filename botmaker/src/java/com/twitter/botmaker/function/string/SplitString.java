package com.twitter.botmaker.function.string;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.RegexCache;
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
        "the text",
        "the delimiter"
    },
    returnType = "List<String>",
    description = "Splits the text on the delimiter.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "SplitString(\"abc@xyz.com\", \"@\")"
    }
)
public class SplitString extends FunctionNode2<Runtime, String, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.<Type>of(
          Type.STRING,
          Type.STRING),
      Type.listOf(Type.STRING)
  );

  public SplitString(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, String text, String sep) {
    final RegexCache regexCache;
    if (context.getRuntime() instanceof RegexCache) {
      regexCache = (RegexCache) (context.getRuntime());
    } else {
      regexCache = RegexCache.DEFAULT;
    }

    return regexCache.split(context, sep, text);
  }
}
