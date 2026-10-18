package com.twitter.botmaker.function.collection;

import java.util.List;
import java.util.Map;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    name = "Map",
    argTypes = {"Collection<Pair<A, B>>"},
    arguments = {"key / value"},
    returnType = "Map<A, B>",
    description = "Makes a list of EVEN number of objects into a map , 0 param is allowed",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Map(123 : \"asdf\", 456 : \"jkl;\")"
    }
)
public class MapOperator extends FunctionNode<Runtime> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.pairOf(TPA, TPB),
      Type.mapOf(TPA, TPB)
  );

  public MapOperator(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(
      Context<Runtime> context,
      List<Object> args
  ) throws Exception {
    Map<Object, Object> hashMap = Maps.newHashMap();
    for (Object arg : args) {
      Pair<?, ?> pair = (Pair<?, ?>) arg;
      Object key = pair.getFirst();
      Object value = pair.getSecond();
      if (key != null && value != null) {
        hashMap.put(key, value);
      }
    }
    return ImmutableMap.copyOf(hashMap);
  }
}
