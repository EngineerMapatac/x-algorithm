package com.twitter.botmaker.function.string;

import java.util.List;

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
    argTypes = {"String", "List<Object>"},
    arguments = {"format template", "values to be embedded"},
    returnType = "String",
    description = "format the string, just like Java Formatter",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "StringFormat(\"{\\\"name\\\": %s, \\\"userId\\\": %d}\", [\"\\\"hello\\\"\", 123456])"
    }
)
public class StringFormat extends FunctionNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.listOf(Type.OBJECT)),
      Type.STRING
  );

  public StringFormat(
      String exprText,
      ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {
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
  protected String apply(Context<Runtime> context, List<Object> args) {
    String template = (String) args.get(0);
    Object[] valueArray = ((List<?>) args.get(1)).toArray();
    return String.format(template, valueArray);
  }
}
