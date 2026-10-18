package com.twitter.botmaker.function.string;

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
    argTypes = {
        "String",
        "String",
        "[Long]"
    },
    arguments = {
        "the input string ",
        "the substring to search for.",
        "from index"
    },
    returnType = "Long",
    actionLevel = ActionLevel.NO_ACTION,
    description =
        "Returns the index within this string of the last occurrence of the specified substring."
)
public class LastIndexOf extends FunctionNode2O1<Runtime, String, String, Long> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      ImmutableList.of(Type.LONG),
      Type.LONG
  );

  public LastIndexOf(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected Long apply(Context<Runtime> context, String str, String substr) {
    if (str == null || substr == null) {
      return -1L;
    }
    return (long) str.lastIndexOf(substr);
  }

  @Override
  protected Long apply(
      Context<Runtime> context,
      String str,
      String substr,
      Long index
  ) {
    if (str == null || substr == null) {
      return -1L;
    }
    return (long) str.lastIndexOf(substr, index.intValue());
  }
}
