package com.twitter.botmaker.function.map;

import java.util.Map;
import java.util.NoSuchElementException;

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
    argTypes = {
        "Map<A, B>",
        "A"
    },
    arguments = {
        "dict",
        "key"
    },
    returnType = "B",
    deprecated = true,
    description = "Checks if the map contains the give key",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "MapContains(Map(123 : \"asdf\", 456 : \"jkl;\"), 123)"
    }
)
public class MapContains extends FunctionNode2<Runtime, Map, Object> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.mapOf(TPA, TPB), TPA),
      Type.BOOLEAN);

  private static final NoSuchElementException EXCEPTION = new NoSuchElementException();

  public MapContains(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Map map, Object key) {
    return map.containsKey(key);
  }
}
