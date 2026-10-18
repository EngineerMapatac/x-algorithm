package com.twitter.botmaker.function.collection;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Sets;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    name = {"Intersection", "SetIntersect", "SetIntersection"},
    argTypes = {"[Set<T>...]"},
    arguments = {
        "the first set",
        "the second set"
    },
    returnType = "Set<T>",
    description = "The intersection of the sets.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Intersection(SetUnion(Set(1, \"x\"), Set(\"y\")), ToSet([1, 2, \"x\"]))",
        "SetIntersect(Set(5, 6, 7, 8), Set(8, 100, 6, 5), Set(5, 7, 233, 8))"
    }
)
public class SetIntersect extends FunctionNode<Runtime> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPS = Type.setOf(TPA);
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      TPS,
      TPS);

  public SetIntersect(String exprText, ImmutableList<ASTNode> children)
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
    Set<Object> result = ImmutableSet.of();
    for (int index = 0; index < args.size(); ++index) {
      @SuppressWarnings("unchecked")
      Set<Object> set = (Set) args.get(index);
      result = (index == 0) ? set : Sets.intersection(result, set);
    }
    return Collections.unmodifiableSet(new HashSet<>(result));
  }
}
