package com.twitter.botmaker.function.math;

import java.util.List;
import java.util.NoSuchElementException;

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
    argTypes = {"Collection<Number>"},
    arguments = {"a list of comparable elements"},
    returnType = "Number",
    description = "Returns the minimum element in the list.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Min(List(1.0, 2.0, 3.0))"
    }
)
public class Min extends FunctionNode1<Runtime, List<Number>> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.collectionOf(Type.NUMBER)),
      Type.NUMBER
  );

  public Min(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, List<Number> numbers) {
    Number min = null;
    for (Number num : numbers) {
      if (min == null || num.doubleValue() < min.doubleValue()) {
        min = num;
      }
    }
    if (min == null) {
      throw new NoSuchElementException("collection is empty");
    }
    return min;
  }
}
