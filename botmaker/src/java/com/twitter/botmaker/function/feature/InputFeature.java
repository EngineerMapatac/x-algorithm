package com.twitter.botmaker.function.feature;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.MissingFeatureException;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public abstract class InputFeature extends ASTNode<Runtime> {

  private InputFeature(String exprText) throws SemanticCheckFailure {
    super(exprText, ImmutableList.<ASTNode>of());
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Event;
  }

  public static InputFeature of(String featureName, Type featureType) throws SemanticCheckFailure {
    final Signature signature = new Signature(ImmutableList.of(), featureType);

    return new InputFeature(featureName) {
      private final MissingFeatureException cachedException = new
          MissingFeatureException(getExprText());

      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      public Extractor<Runtime> toExtractor() {
        return new Functor<Runtime>(this, ImmutableList.of()) {

          @Override
          public Future<Object> run(Context<Runtime> context) {
            return Future.apply(Function.exfunc0(() -> apply(context)));
          }

          @Override
          public Object apply(Context<Runtime> context) {
            if (!context.isInputFeatureDefined(getExprText())) {
              throw new FunctionFailure(getAstNode(), context.getStackFrames(), cachedException);
            }
            Object featureValue = context.getInputFeature(getExprText());
            return featureValue;
          }
        };
      }
    };
  }
}
