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
    description = "Returns the maximum element in the list.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Max(List(1,998,42,3))"
    }
)
public class Max extends FunctionNode1<Runtime, List<Number>> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.collectionOf(Type.NUMBER)),
      Type.NUMBER
  );

  public Max(String exprText, ImmutableList<ASTNode> children)
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
    Number max = null;
    for (Number num : numbers) {
      if (max == null || max.doubleValue() < num.doubleValue()) {
        max = num;
      }
    }
    if (max == null) {
      throw new NoSuchElementException("collection is empty");
    }
    return max;
  }
}
