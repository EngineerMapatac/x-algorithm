package com.twitter.botmaker.function.bitwise;

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
    argTypes = {"Long"},
    arguments = {"A"},
    returnType = "Long",
    description = "Returns bitwise (~A).",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "BitwiseInvert(-1)"
    }
)
public class BitwiseInvert extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG),
      Type.LONG
  );

  public BitwiseInvert(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, List<Object> args) {
    Long a = (Long) args.get(0);

    return ~a;
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }
}
