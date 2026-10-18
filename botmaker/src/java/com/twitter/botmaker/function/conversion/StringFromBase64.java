package com.twitter.botmaker.function.conversion;

import java.util.Base64;

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
    arguments = {"Base64-encoded string"},
    returnType = "String",
    description = "Returns the decoded string.",
    actionLevel = ActionLevel.NO_ACTION
)
public class StringFromBase64 extends FunctionNode1<Runtime, String> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
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

  public StringFromBase64(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, String input) {
    return new String(Base64.getDecoder().decode(input));
  }
}
