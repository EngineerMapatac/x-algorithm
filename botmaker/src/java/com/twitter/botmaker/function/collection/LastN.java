package com.twitter.botmaker.function.collection;

import java.util.List;

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
    argTypes = {"List<T>", "Long"},
    arguments = {"the list", "the number of trailing items to return"},
    returnType = "List<T>",
    description =
        "Returns the last n items of the list."
            + " Returns the whole list if n is greater than the length of the list.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "LastN(List(\"asdf\", \"bas\", \"biz\"), 2)"
    }
)
public class LastN extends FunctionNode2<Runtime, List<Object>, Long> {
  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.listOf(TP), Type.LONG),
      Type.listOf(TP));

  public LastN(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Object apply(Context<Runtime> context, List<Object> list, Long n) {
    int fromIndex = list.size() - n.intValue();
    fromIndex = Math.max(fromIndex, 0);
    fromIndex = Math.min(fromIndex, list.size());
    return list.subList(fromIndex, list.size());
  }
}
