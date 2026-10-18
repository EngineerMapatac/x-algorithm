package com.twitter.botmaker.function.misc;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.ClassCache;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;


@BotMakerFunction(
    argTypes = {"Object", "String"},
    arguments = {"input object", "class name"},
    returnType = "Boolean",
    description = "Return whether the given input object is an instance"
        + " of the class given by the class name. Follows Java instanceOf check rules. If the"
        + " class name passed in does not exist, will throw an Exception.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {"InstanceOf(:str, \"java.lang.String\")"}
)

public class InstanceOf extends FunctionNode2<Runtime, Object, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.OBJECT, Type.STRING),
      Type.BOOLEAN
  );

  public InstanceOf(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  public Boolean apply(Context<Runtime> context, Object inputObject, String clazzName)
      throws Exception {
    Class<?> classToCheck = ClassCache.forName(clazzName);
    return classToCheck.isInstance(inputObject);
  }
}
