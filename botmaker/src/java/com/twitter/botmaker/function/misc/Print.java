package com.twitter.botmaker.function.misc;

import java.util.logging.Level;
import java.util.logging.Logger;

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
    argTypes = {"Object"},
    arguments = {"object to print"},
    returnType = "Unit",
    description = "Log an object.",
    actionLevel = ActionLevel.NO_ACTION
)

public class Print extends FunctionNode1<Runtime, Object> {

  private static final Logger LOGGER = Logger.getLogger(Print.class.getName());

  private static final ASTNode.Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT),
      Type.UNIT
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Event;
  }

  public Print(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, Object info) {
    LOGGER.log(Level.INFO, "bot/" + context.getTrackingId() + " log: " + String.valueOf(info));
    return unit();
  }
}
