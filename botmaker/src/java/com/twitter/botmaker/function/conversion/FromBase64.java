package com.twitter.botmaker.function.conversion;

import java.nio.ByteBuffer;
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
    arguments = {"Base64 string to decode"},
    returnType = "Binary",
    description = "Decode a base64 string to binary.",
    actionLevel = ActionLevel.NO_ACTION
)
public class FromBase64 extends FunctionNode1<Runtime, Object> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      Type.BINARY
  );

  public FromBase64(String exprText, ImmutableList<ASTNode> children)
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
    return ByteBuffer.wrap(
        Base64.getDecoder().decode((String) arg)
    );
  }
}
