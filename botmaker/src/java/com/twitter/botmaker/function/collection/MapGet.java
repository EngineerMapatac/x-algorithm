package com.twitter.botmaker.function.collection;

import java.util.Map;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2O1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    name = {"MapGet", "GetMapValue", "DictGetOrElse"},
    argTypes = {
        "Map<A, B>",
        "A",
        "B"
    },
    arguments = {
        "dict",
        "key",
        "default value"
    },
    returnType = "B",
    description = "Gets the value of key in the dict or use the default value when missing",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "MapGet(Map(123 : \"asdf\", 456 : \"jkl;\"), 555, \"xxx\")"
    }
)
public class MapGet extends FunctionNode2O1<Runtime, Map, Object, Object> {

  private static final Type TPA = Type.newGenericType();
  private static final Type TPB = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.mapOf(TPA, TPB), TPA),
      ImmutableList.of(TPB),
      TPB);

  public MapGet(String exprText, ImmutableList<ASTNode> children)
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
      Context<Runtime> context, Map map, Object key, Object defaultValue) {
    Object value = map.get(key);
    if (value != null) {
      return value;
    } else {
      return defaultValue;
    }
  }

  @Override
  protected Object apply(
      Context<Runtime> context, Map map, Object key) throws Exception {
    Object value = map.get(key);
    return value;
  }
}
