package com.twitter.botmaker.function.conversion;

import com.google.common.base.Strings;
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
    argTypes = {"Long"},
    arguments = {"Long to convert to a hex"},
    returnType = "String",
    description = "Left padded string hex representation of a given long number",
    actionLevel = ActionLevel.NO_ACTION
)
public class ToHex extends FunctionNode1<Runtime, Number> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER),
      Type.STRING
  );

  public ToHex(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Number arg) {
    String hexString = Long.toHexString(arg.longValue());
    String leftPad = Strings.repeat("0", 16 - hexString.length());
    return leftPad + hexString;
  }
}
