package com.twitter.botmaker.function.collection;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2O1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "Long",
        "Long",
        "[Long]"
    },
    arguments = {
        "start",
        "stop",
        "step"
    },
    returnType = "List<Long>",
    description = "Creates a lists containing arithmetic progressions from start (inclusive)"
        + " to stop (exclusive). If the optional step size argument is omitted, it"
        + " defaults to 1.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Range(0,3)",
        "Range(1,5,2)"
    }
)
public class Range extends FunctionNode2O1<Runtime, Long, Long, Long> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG, Type.LONG),
      ImmutableList.of(Type.LONG),
      Type.listOf(Type.LONG)
  );

  public Range(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(
      Context<Runtime> context,
      Long start,
      Long stop,
      Long step
  ) {
    ImmutableList.Builder<Long> builder = new ImmutableList.Builder<>();

    for (long i = start; i < stop; i += step) {
      builder.add(i);
    }

    return builder.build();
  }

  @Override
  protected Object apply(
      Context<Runtime> context,
      Long start,
      Long stop
  ) {
    return apply(context, start, stop, 1L);
  }

}
