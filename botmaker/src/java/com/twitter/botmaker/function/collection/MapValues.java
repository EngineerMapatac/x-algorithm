package com.twitter.botmaker.function.collection;

import java.util.Map;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Sets;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    name = {"GetMapValues", "MapValues"},
    argTypes = {"Map<A, B>"},
    arguments = {"the map whose values to get"},
    returnType = "Set<B>",
    actionLevel = ActionLevel.NO_ACTION,
    description = "Get values of a map"
)
public class MapValues extends FunctionNode1<Runtime, Map<Object, Object>> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.mapOf(TPA, TPB)),
      Type.setOf(TPB)
  );

  public MapValues(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Map<Object, Object> map) {
    return Sets.newHashSet(map.values());
  }
}
