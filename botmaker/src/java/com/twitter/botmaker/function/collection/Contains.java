package com.twitter.botmaker.function.collection;

import java.util.Collection;
import java.util.Map;
import java.util.Set;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.function.conversion.ToSet;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "Object",
        "Object"
    },
    arguments = {
        "the collection",
        "the element"
    },
    returnType = "Boolean",
    description = "Whether the given collection contains the given element.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Contains([1, 2, 3], 2)"
    }
)
public class Contains extends FunctionNode2<Runtime, Object, Object> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(
          Type.OBJECT,
          Type.OBJECT),
      Type.BOOLEAN
  );

  public Contains(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, validateASTNodesAndModifyType(children));
  }

  private static ImmutableList<ASTNode> validateASTNodesAndModifyType(
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    Class<?> typeBase = children.get(0).getReturnType().typeBase;
    if (!Object.class.equals(typeBase)
        && !String.class.equals(typeBase)
        && !Collection.class.isAssignableFrom(typeBase)
        && !Map.class.isAssignableFrom(typeBase)) {
      throw new SemanticCheckFailure(String.format(
          "Contains expects input of String, Map, Set or Collection types but %s received",
          typeBase.getName()));
    }

    if (Collection.class.isAssignableFrom(typeBase) && !Set.class.isAssignableFrom(typeBase)) {
      ASTNode setifiedCollection = new ToSet("ToSet", ImmutableList.of(children.get(0)));
      return ImmutableList.of(setifiedCollection, children.get(1));
    }
    return children;
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
      Object container,
      Object value
  ) {
    if (container instanceof String) {
      return ((String) container).contains(value.toString());
    } else if (container instanceof Collection) {
      return ((Collection) container).contains(value);
    } else if (container instanceof Map) {
      return ((Map) container).containsKey(value);
    }
    return false;
  }
}
