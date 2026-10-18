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
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"[List<T>...]"},
    arguments = {"0 or more elements"},
    returnType = "List<T>",
    description = "Concatenates two or more lists into a single list",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ListConcat(List(\"asdf\", \"bas\"), List(\"mik\", \"zgoot\"))"
    }
)
public class ListConcat extends FunctionNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.collectionOf(TP),
      Type.listOf(TP)
  );

  public ListConcat(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, List<Object> args) {
    int resultSize = args.stream().mapToInt(arg -> ((Collection<?>) arg).size()).sum();
    List<Object> resultList = new ArrayList<>(resultSize);
    for (Object arg : args) {
      @SuppressWarnings("unchecked")
      Collection<Object> listArg = (Collection<Object>) arg;
      resultList.addAll(listArg);
    }
    return Collections.unmodifiableList(resultList);
  }
}
