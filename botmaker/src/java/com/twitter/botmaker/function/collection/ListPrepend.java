package com.twitter.botmaker.function.collection;

import java.util.Collection;
import java.util.Collections;
import java.util.List;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {
        "List<T>",
        "[T]*"
    },
    arguments = {
        "The list to be prepended",
        "items to prepend to the list"
    },
    returnType = "List<T>",
    description = "Prepend items to a list",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ListJoin(\", \", List(2, \"name\"))"
    }
)
public class ListPrepend extends FunctionNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.listOf(TP)),
      TP,
      Type.listOf(TP)
  );

  public ListPrepend(String exprText, ImmutableList<ASTNode> children)
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
    Collection<?> collection = (Collection<?>) args.get(0);
    List<Object> list = Lists.newArrayListWithCapacity(collection.size() + args.size() - 1);
    for (int i = 1; i < args.size(); i++) {
      list.add(args.get(i));
    }
    list.addAll(collection);
    return Collections.unmodifiableList(list);
  }
}

