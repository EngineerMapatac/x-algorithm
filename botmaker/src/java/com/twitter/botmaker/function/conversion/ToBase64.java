package com.twitter.botmaker.function.conversion;

import java.nio.ByteBuffer;
import java.util.Base64;

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
    arguments = {"object to convert to a base64 string"},
    returnType = "String",
    description = "Base64 String representation of the object.",
    actionLevel = ActionLevel.NO_ACTION
)
public class ToBase64 extends FunctionNode1<Runtime, Object> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(TP),
      Type.STRING
  );

  private final Serializer serializer;

  public ToBase64(String exprText, ImmutableList<ASTNode> children)
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
    ByteBuffer bytes = serializer.serialize(arg, false);
    return Base64.getEncoder().encodeToString(bytes.array());
  }
}
