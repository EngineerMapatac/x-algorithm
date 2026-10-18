package com.twitter.botmaker.function.misc;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode0;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"T"},
    arguments = {"input object"},
    returnType = "String",
    description = "Get type name of the object.",
    actionLevel = ActionLevel.NO_ACTION
)

public class TypeName extends FunctionNode0<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.STRING
  );

  private final String typeName;

  public TypeName(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, ImmutableList.of());

    assertChildrenSize(exprText, children, 1);
    typeName = children.get(0).getReturnType().toString();
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
  protected Object apply(Context<Runtime> context) {
    return typeName;
  }
}
