package com.twitter.botmaker.function.string;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"[String...]"},
    arguments = {"the strings to concatenate"},
    returnType = "String",
    description = "Concatenates multiple strings",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Concat(\"Bot\", \"Maker\")",
        "Concat(\"Bot\", \"Maker\", Concat(\" rocks\", \" hard!\"))"
    }
)
public class Concat extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.<Type>of(),
      Type.OBJECT,
      Type.STRING
  );

  public Concat(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected Object apply(Context<Runtime> context, List<Object> args) {
    StringBuilder stringBuilder = new StringBuilder();
    for (Object str : args) {
      stringBuilder.append(str);
    }
    return stringBuilder.toString();
  }
}
