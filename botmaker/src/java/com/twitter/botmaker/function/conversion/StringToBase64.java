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
    arguments = {"String to encode"},
    returnType = "String",
    description = "Returns the base64-encoded string.",
    actionLevel = ActionLevel.NO_ACTION
)
public class StringToBase64 extends FunctionNode1<Runtime, String> {
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

  public StringToBase64(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, String input) {
    return Base64.getEncoder().encodeToString(input.getBytes());
  }
}
