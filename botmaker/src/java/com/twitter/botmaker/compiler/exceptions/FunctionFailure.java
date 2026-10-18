package com.twitter.botmaker.compiler.exceptions;

import java.util.List;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.tuples.StackFrame;

public class FunctionFailure extends RuntimeFailure {

  public final ASTNode astNode;
  public final List<StackFrame> stackFrames;

  public FunctionFailure(
      final ASTNode astNode,
      final List<StackFrame> stackFrames,
      final Throwable t
  ) {
    super(astNode.getExprText(), t);
    this.astNode = astNode;
    this.stackFrames = stackFrames;
  }

  public String getStackFrameMessage() {
    StringBuilder builder = new StringBuilder();

    for (StackFrame stackFrame : stackFrames) {
      builder.append("    at ")
          .append('(')
          .append(stackFrame.getExpr())
          .append(':')
          .append(stackFrame.getLineInParentFrame())
          .append(')')
          .append('\n');
    }
    return builder.toString();
  }

  @Override
  public Throwable fillInStackTrace() {
    return this;
  }

  public static Throwable unwrap(Throwable th) {
    if (th instanceof FunctionFailure) {
      return th.getCause();
    }
    return th;
  }

  public static FunctionFailure wrap(Throwable th, ASTNode<?> astNode, Context<?> context) {
    if (th instanceof FunctionFailure) {
      return (FunctionFailure) th;
    }
    return new FunctionFailure(astNode, context.getStackFrames(), th);
  }

}
