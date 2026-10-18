package com.twitter.botmaker.function.conversion;

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
    arguments = {"Hex String to convert to a Long"},
    returnType = "Long",
    description = "Parses from hex into a Long",
    actionLevel = ActionLevel.NO_ACTION
)
public class FromHex extends FunctionNode1<Runtime, String>  {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      Type.LONG
  );

  public FromHex(
      String exprText, ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {
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
  protected Object apply(Context<Runtime> context, String input) throws Exception {
    return Long.parseUnsignedLong(input, 16);
  }
}
