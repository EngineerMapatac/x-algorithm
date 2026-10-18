package com.twitter.botmaker.function.feature;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Future;

public abstract class GetOrElse extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(ImmutableList.of(TP, TP), TP);

  private final String inputFeatureName;

  public GetOrElse(
      String exprText,
      String inputFeatureName,
      ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {

    super(exprText, children);
    this.inputFeatureName = inputFeatureName;
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
  public Extractor<Runtime> toExtractor() {

    final ImmutableList<Extractor> childObjectExtractors = buildExtractorsOfChildren();
    final Extractor elseExpr = childObjectExtractors.get(1);

    if (isFunctor(childObjectExtractors)) {
      return new Functor<Runtime>(this, childObjectExtractors) {

        @Override
        public Future run(Context<Runtime> context) {
          if (context.isInputFeatureDefined(inputFeatureName)) {
            return Future.value(context.getInputFeature(inputFeatureName));
          } else {
            return context.run(elseExpr);
          }
        }

        @Override
        public Object apply(Context<Runtime> context) throws Exception {
          if (context.isInputFeatureDefined(inputFeatureName)) {
            return context.getInputFeature(inputFeatureName);
          } else {
            return ((Functor) elseExpr).apply(context);
          }
        }
      };
    } else {
      return new Extractor<Runtime>(this, childObjectExtractors) {

        @Override
        public Future run(Context<Runtime> context) {
          if (context.isInputFeatureDefined(inputFeatureName)) {
            return Future.value(context.getInputFeature(inputFeatureName));
          } else {
            return context.run(elseExpr);
          }
        }
      };
    }
  }

  public static GetOrElse of(
      String exprText,
      String inputFeatureName,
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    return new GetOrElse(exprText, inputFeatureName, children) {
      @Override
      protected void computeFingerprint(Hasher hasher) {
        super.computeFingerprint(hasher);
        hasher.putUnencodedChars(inputFeatureName);
      }
    };
  }
}
