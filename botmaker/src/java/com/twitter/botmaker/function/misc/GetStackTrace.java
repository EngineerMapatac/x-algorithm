package com.twitter.botmaker.function.misc;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.StackFrame;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode0;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {},
    arguments = {},
    returnType =
        "List of StackFrame(expr, lineInParentFrame) tuples as the stacktrace of the ASTNode",
    description = "Get StackTrace of the ASTNode.",
    actionLevel = ActionLevel.NO_ACTION
)
public class GetStackTrace extends FunctionNode0<Runtime> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.listOf(StackFrame.TYPE)
  );

  public GetStackTrace(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Event;
  }

  @Override
  protected Object apply(Context<Runtime> context) {
    return context.getStackFrames();
  }
}
