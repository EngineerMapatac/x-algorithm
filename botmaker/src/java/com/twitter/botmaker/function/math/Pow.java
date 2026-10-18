package com.twitter.botmaker.function.math;

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
    argTypes = {
        "Number",
        "Number"
    },
    arguments = {
        "number base",
        "number exponent"
    },
    returnType = "Double",
    description = "Returns one number raised to the power of another.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Pow(2, 3)"
    }
)
public class Pow extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER, Type.NUMBER),
      Type.DOUBLE
  );

  public Pow(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Double apply(Context<Runtime> context, List<Object> args) {
    double base = ((Number) args.get(0)).doubleValue();
    double exponent = ((Number) args.get(1)).doubleValue();

    return Math.pow(base, exponent);
  }
}
