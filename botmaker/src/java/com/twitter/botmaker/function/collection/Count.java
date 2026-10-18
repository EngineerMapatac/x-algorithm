package com.twitter.botmaker.function.collection;

import java.util.Collection;
import java.util.Map;

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
    name = {"Length", "Count", "Size"},
    argTypes = {"Collection/String/Map"},
    arguments = {"a string, map. or collection"},
    returnType = "Long",
    description = "The size of the given object.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Count(ToSet(List(1, 2, 3)))"
    }
)
public class Count extends FunctionNode1<Runtime, Object> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT),
      Type.LONG
  );

  public Count(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);

    Class<?> typeBase = children.get(0).getReturnType().typeBase;
    if (!Object.class.equals(typeBase)
        && !String.class.equals(typeBase)
        && !Collection.class.isAssignableFrom(typeBase)
        && !Map.class.isAssignableFrom(typeBase)) {
      throw new SemanticCheckFailure(String.format(
          "Count expects input of String, Map or Collection types but %s received",
          typeBase.getName()));
    }

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
  @SuppressWarnings("unchecked")
  protected Object apply(Context<Runtime> context, Object arg) {
    if (arg instanceof String) {
      String str = (String) arg;
      return (long) str.length();
    } else if (arg instanceof Map) {
      Map<?, ?> map = (Map<?, ?>) arg;
      return (long) map.size();
    } else {
      Collection<?> collection = (Collection<?>) arg;
      return (long) collection.size();
    }
  }
}
