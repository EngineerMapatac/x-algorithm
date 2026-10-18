package com.twitter.botmaker.function.logic;

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
    internal = true,
    argTypes = {},
    arguments = {},
    returnType = "Boolean",
    description = "Internal use only.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "a == b != c"
    }
)
public class NotEquals extends FunctionNode2<Runtime, Object, Object> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT, Type.OBJECT),
      Type.BOOLEAN
  );

  public NotEquals(String exprText, ImmutableList<ASTNode> children)
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
  public Object apply(Context<Runtime> context, Object a, Object b) {
    if (a == null || b == null) {
      return a != b;
    } else if (a instanceof Number && b instanceof Number) {
      Number numA = (Number) a;
      Number numB = (Number) b;

      if (Type.doubleLike(numA) || Type.doubleLike(numB)) {
        return numA.doubleValue() != numB.doubleValue();
      } else {
        return numA.longValue() != numB.longValue();
      }
    } else {
      return !a.equals(b);
    }
  }
}
