package com.twitter.botmaker.function.feature;

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
    argTypes = {"String", "T"},
    arguments = {"feature name", "feature value"},
    returnType = "T",
    description = "produce an output feature.",
    actionLevel = ActionLevel.NO_ACTION
)
public class OutputFeature extends FunctionNode2<Runtime, String, Object> {

  public static final Object NAMESPACE = new Object() {
    @Override
    public String toString() {
      return "OutputFeature";
    }
  };

  private static final Type TP = Type.newGenericType();

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, TP),
      TP
  );

  public OutputFeature(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Event;
  }

  @Override
  protected Object apply(Context<Runtime> context, String featureName, Object featureValue) {
    context.addOutputFeature(
        NAMESPACE,
        featureName,
        getChildren().get(1).getReturnType(),
        featureValue);
    return featureValue;
  }
}
