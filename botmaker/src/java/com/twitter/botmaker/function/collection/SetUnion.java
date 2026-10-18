package com.twitter.botmaker.function.collection;

import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

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
    argTypes = {"[Set<T>...]"},
    arguments = {"sets"},
    returnType = "Set<T>",
    description = "The union of the sets.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "SetUnion(ToSet([1, 2]), Set(2, 3, 4))"
    }
)
public class SetUnion extends FunctionNode<Runtime> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPS = Type.setOf(TPA);
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      TPS,
      TPS);

  public SetUnion(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, List<Object> args) {
    Set<Object> resultSet = new HashSet<>();
    for (Object arg : args) {
      resultSet.addAll((Collection<?>) arg);
    }
    return Collections.unmodifiableSet(resultSet);
  }
}
