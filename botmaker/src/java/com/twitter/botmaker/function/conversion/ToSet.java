package com.twitter.botmaker.function.conversion;

import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

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
    argTypes = {"Collection<T>"},
    arguments = {"a list of elements"},
    returnType = "Set<T>",
    description = "Returns a new set containing the elements in the given list.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ToSet(flatmap (:a in [Set(\"a\", \"b\", \"c\"), [\"d\"]]) { :a })"
    }
)
public class ToSet extends FunctionNode1<Runtime, Collection<Object>> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.collectionOf(TP)),
      Type.setOf(TP)
  );

  public ToSet(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, Collection<Object> collection) {
    if (collection instanceof Set
        && collection.getClass().getSimpleName().equals("UnmodifiableSet")) {
      return collection;
    }
    return Collections.unmodifiableSet(new HashSet<>(collection));
  }
}
