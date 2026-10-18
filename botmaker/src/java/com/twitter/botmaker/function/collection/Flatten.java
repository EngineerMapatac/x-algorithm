package com.twitter.botmaker.function.collection;

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
    argTypes = {"Collection<Collection<T>>"},
    arguments = {"a collection of collections of elements"},
    returnType = "List<T>",
    description = "Converts collection of collections into a list formed by the elements of"
        + " these collections.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "flatmap (:a in [1,2,3]) { [ToString(:a), :a * 2.0] }"
    }
)
public class Flatten extends FunctionNode1<Runtime, Collection<Collection<Object>>> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.collectionOf(Type.collectionOf(TP))),
      Type.listOf(TP));

  public Flatten(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Object apply(
      Context<Runtime> context,
      Collection<Collection<Object>> cc
  ) {
    int resultSize = cc.stream().mapToInt(Collection::size).sum();
    List<Object> resultList = new ArrayList<>(resultSize);
    for (Collection<Object> c : cc) {
      resultList.addAll(c);
    }
    return Collections.unmodifiableList(resultList);
  }
}
