package com.twitter.botmaker.function.math;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "Long",
        "Long"
    },
    arguments = {
        "A",
        "B"
    },
    returnType = "Long",
    description = "Returns A % B",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Mod(5, 2)"
    }
)
public class Mod extends FunctionNode2<Runtime, Long, Long> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG, Type.LONG),
      Type.LONG
  );

  public Mod(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Long numA, Long numB) {
    if (numB == 0L) {
      return 0L;
    }
    return numA % numB;
  }
}
