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
    argTypes = {"Object"},
    arguments = {"object to convert to a BotMaker script"},
    returnType = "String",
    description = "Get the BotMaker script representation of the object.",
    actionLevel = ActionLevel.NO_ACTION
)
public class ToScript extends FunctionNode1<Runtime, Object> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT),
      Type.STRING
  );

  public ToScript(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Object arg) throws Exception {
    return Type.OBJECT.serializer.getScript(arg);
  }
}
