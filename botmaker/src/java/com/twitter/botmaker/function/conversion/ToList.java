package com.twitter.botmaker.function.conversion;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;

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
    argTypes = {"Collection<T>"},
    arguments = {"a collection of elements"},
    returnType = "List<T>",
    description = "Returns a new list containing the elements in the given collection.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ToList(Set(1,2,3))"
    }
)
public class ToList extends FunctionNode1<Runtime, Collection<Object>> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.collectionOf(TP)),
      Type.listOf(TP)
  );

  public ToList(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Collection<Object> collection) {
    if (collection instanceof List) {
      return collection;
    } else {
      return Collections.unmodifiableList(new ArrayList<>(collection));
    }
  }
}
