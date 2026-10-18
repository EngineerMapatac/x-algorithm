package com.twitter.botmaker.function.collection;

import java.util.Collection;

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
    argTypes = {
        "Object"
    },
    arguments = {
        "the input object to be checked"
    },
    returnType = "Boolean",
    description = "Checks if the object is a collection",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "IsCollection([1, 2, 3])",
        "IsCollection(22)"
    }
)
public class IsCollection extends FunctionNode1<Runtime, Object> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT),
      Type.BOOLEAN);

  public IsCollection(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Object obj) {
    return obj instanceof Collection;
  }
}
