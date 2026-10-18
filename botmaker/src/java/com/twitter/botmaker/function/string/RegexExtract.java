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
    name = {
        "RegexExtract",
        "RegexExtractGroups"
    },
    argTypes = {
        "String",
        "String"
    },
    arguments = {
        "The regex with groups",
        "The target string"
    },
    returnType = "List<String>",
    description = "Example: RegexExtract(\"(?<text>\\w+) (?<numbers>\\d+)\", \"hello"
        + " 123\") returns the list: [hello, 123]\n",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "RegexExtract(\"(?<groupName1>\\w+) (?<groupName2>\\d+)\", \"TEST"
            + " 123\")"
    }
)
public class RegexExtract extends FunctionNode2<Runtime, String, String> {
  private static final ASTNode.Signature SIGNATURE = new ASTNode.Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      Type.listOf(Type.STRING)
  );

  public RegexExtract(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public ASTNode.Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  public Object apply(Context<Runtime> context, String regex, String str) {

    final RegexCache regexCache;
    if (context.getRuntime() instanceof RegexCache) {
      regexCache = (RegexCache) (context.getRuntime());
    } else {
      regexCache = RegexCache.DEFAULT;
    }

    return regexCache.extract(context, regex, str);
  }
}
