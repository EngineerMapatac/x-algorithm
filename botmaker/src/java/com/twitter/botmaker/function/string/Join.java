package com.twitter.botmaker.function.string;

import java.util.Iterator;
import java.util.List;

import com.google.common.base.Joiner;
import com.google.common.collect.ImmutableList;

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
        "String",
        "[Object...]"
    },
    arguments = {
        "the delimiter to use",
        "the objects to concatenate"
    },
    returnType = "String",
    description = "Concatenates multiple objects with the given delimiter",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Join(\", \", 2, \"name\")"
    }
)
public class Join extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      Type.OBJECT,
      Type.STRING
  );

  public Join(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  public Object apply(Context<Runtime> context, List<Object> args) {
    Iterator<Object> iterator = args.iterator();
    return Joiner.on((String) iterator.next()).join(iterator);
  }
}
