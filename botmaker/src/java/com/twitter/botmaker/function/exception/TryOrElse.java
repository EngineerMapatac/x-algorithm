package com.twitter.botmaker.function.exception;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

@BotMakerFunction(
    argTypes = {
        "T",
        "T"
    },
    arguments = {
        "expression",
        "backup-expression"
    },
    returnType = "T",
    description = "TryOrElse ensure function won't fail due to exception. ( assuming the"
        + " backup expression will always run successfully)\n"
        + "If the first expression run successfully, return its value. \n"
        + "If the first expression throws exception, return the value of second"
        + " expression\n"
        + "",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "TryOrElse(TRUE, FALSE)"
    }
)
public class TryOrElse extends ASTNode<Runtime> {


  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(TP, TP),
      TP
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public TryOrElse(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Extractor<Runtime> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();


    if (isFunctor(childExtractors)) {
      final Functor tryFunctor = (Functor) childExtractors.get(0);
      final Functor elseFunctor = (Functor) childExtractors.get(1);

      return new Functor<Runtime>(this, childExtractors) {

        @Override
        public Object apply(Context<Runtime> context) throws Exception {
          try {
            return tryFunctor.apply(context);
          } catch (Exception ex) {
            return elseFunctor.apply(context);
          }
        }

        @Override
        public Future<Object> run(Context<Runtime> context) {
          return TryOrElse.this.run(context, tryFunctor, elseFunctor);
        }
      };
    } else {

      final Extractor tryExtractor = childExtractors.get(0);
      final Extractor elseExtractor = childExtractors.get(1);

      return new Extractor<Runtime>(this, childExtractors) {

        @Override
        public Future run(final Context<Runtime> context) {
          return TryOrElse.this.run(context, tryExtractor, elseExtractor);
        }
      };
    }
  }

  private Future run(Context<Runtime> context, Extractor tryExtractor, Extractor elseExtractor) {
    return context.run(tryExtractor).rescue(Function.func(t -> context.run(elseExtractor)));
  }

}
