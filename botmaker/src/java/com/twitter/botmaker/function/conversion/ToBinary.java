package com.twitter.botmaker.function.conversion;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.Serializer;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"Object"},
    arguments = {"object to convert to a bytebuffer"},
    returnType = "Binary",
    description = "Serialize an object to a bytebuffer.",
    actionLevel = ActionLevel.NO_ACTION

)
public class ToBinary extends FunctionNode1<Runtime, Object> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(TP),
      Type.BINARY
  );

  private final Serializer serializer;

  public ToBinary(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);

    this.serializer = children.get(0).getReturnType().serializer;
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
    return serializer.serialize(arg, false);
  }
}
