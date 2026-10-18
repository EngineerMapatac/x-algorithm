package com.twitter.botmaker.function.collection;

import java.util.List;

import com.google.common.base.Joiner;
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
    name = {
        "ListJoin",
        "JoinList"
    },
    argTypes = {
        "String",
        "List<T>"
    },
    arguments = {
        "String the delimiter to use",
        "List<T> the list of objects to concatenate"
    },
    returnType = "String",
    description = "Concatenates the ToString of each element in the given List with the given"
        + " delimiter. Same functionality as Join  but takes a list rather than a"
        + " variable number of Objects.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "ListJoin(\", \", List(2, \"name\"))"
    }
)
public class ListJoin extends FunctionNode2<Runtime, String, List<Object>> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.listOf(TP)),
      Type.STRING
  );

  public ListJoin(String exprText, ImmutableList<ASTNode> children)
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
  protected Object apply(Context<Runtime> context, String sep, List<Object> list) {
    return Joiner.on(sep).join(((List<?>) list).iterator());
  }

}

