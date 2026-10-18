package com.twitter.botmaker.function.bitwise;

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
        "Long"
    },
    arguments = {
        "long"
    },
    returnType = "Long",
    description = "Returns a new Long produced by reversing the bits of the given Long",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ReverseBits(3)"
    }
)
public class ReverseBits extends FunctionNode1<Runtime, Long> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG),
      Type.LONG
  );

  public ReverseBits(
      String exprText, ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected Object apply(Context<Runtime> context, Long input) throws Exception {
    return Long.reverse(input);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }
}
