package com.twitter.botmaker.function.collection;

import java.util.stream.Collectors;
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
    name = {"Distinct"},
    argTypes = {"List<T>"},
    arguments = {"A list from which to get distinct elements"},
    returnType = "List<T>",
    description = "Returns a deduped, ordered list of unique elements. "
      + "Different than Set which doesn't guarantee order.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Distinct(SplitString(tweetText, ' '))"
    }
)
public class Distinct extends FunctionNode1<Runtime, List<Object>> {

  private static final Type TPA = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.listOf(TPA)),
      Type.listOf(TPA));

  public Distinct(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, List<Object> list) {
    return list.stream().distinct().collect(Collectors.toList());
  }
}
