package com.twitter.botmaker.function.exception;

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
    argTypes = {"Boolean", "[String]"},
    arguments = {"condition", "message"},
    returnType = "Long",
    description = "If the condition is not satisfied, throws an exception with the given message",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Assert(1 != 2, \"One should not be equal to two\")"
    }
)
public class Assert extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.BOOLEAN),
      ImmutableList.of(Type.STRING),
      Type.LONG
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Never;
  }

  public Assert(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Object apply(Context<Runtime> context, List<Object> args) {

    boolean bool = (boolean) args.get(0);
    String message = args.size() > 1 ? (String) args.get(1) : "Assert failed";

    if (!bool) {
      throw new BotmakerAssertionException(message);
    }

    return 0L;
  }

  private static final class BotmakerAssertionException extends RuntimeException {

    public BotmakerAssertionException(String message) {
      super(message);
    }

    @Override public synchronized Throwable fillInStackTrace() {
      return this;
    }
  }
}
