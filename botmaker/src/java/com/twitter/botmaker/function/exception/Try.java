package com.twitter.botmaker.function.exception;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

@BotMakerFunction(
    argTypes = {
        "T"
    },
    arguments = {
        "expression"
    },
    returnType = "StructTuple<result:T, exception:String, message:String>",
    description = "Try ensures expression won't fail due to exception.\n"
        + "If the expression runs successfully, return its value as the first element. \n"
        + "If the expression runs successfully and returns null, \n"
        + "then return the null as the first element. \n"
        + "If the expression throws an exception, return the exception type as the second element,"
        + " and the exception message as the third element.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Try(Assert(FALSE, \"error\"))"
    }
)
public class Try extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(TP),
      Type.structTupleOf(
          com.twitter.botmaker.compiler.tuples.Try.FIELD_NAMES,
          ImmutableList.of(TP, Type.STRING, Type.STRING)
      )
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public Try(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Extractor<Runtime> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();


    if (isFunctor(childExtractors)) {
      final Functor tryFunctor = (Functor) childExtractors.get(0);

      return new Functor<Runtime>(this, childExtractors) {

        @Override
        public Object apply(Context<Runtime> context) {
          try {
            return new com.twitter.botmaker.compiler.tuples.Try(
                tryFunctor.apply(context), null, null);
          } catch (FunctionFailure ff) {
            return new com.twitter.botmaker.compiler.tuples.Try(
                null, ff.getCause().getClass().getName(), ff.getCause().getMessage());
          } catch (Exception ex) {
            return new com.twitter.botmaker.compiler.tuples.Try(
                null, ex.getClass().getName(), ex.getMessage());
          }
        }

        @Override
        public Future<Object> run(Context<Runtime> context) {
          return Try.this.run(context, tryFunctor);
        }
      };
    } else {

      final Extractor tryExtractor = childExtractors.get(0);

      return new Extractor<Runtime>(this, childExtractors) {

        @Override
        public Future run(final Context<Runtime> context) {
          return Try.this.run(context, tryExtractor);
        }
      };
    }
  }

  private Future run(Context<Runtime> context, Extractor tryExtractor) {
    return context
        .run(tryExtractor)
        .transform(Function.func((com.twitter.util.Try<Object> trial) -> {
          if (trial.isReturn()) {
            return Future.value(
                new com.twitter.botmaker.compiler.tuples.Try(
                    trial.get(),
                    null,
                    null));
          } else {
            Throwable throwable = FunctionFailure.unwrap(trial.throwable());
            return Future.value(
                new com.twitter.botmaker.compiler.tuples.Try(
                    null,
                    throwable.getClass().getName(),
                    throwable.getMessage()));
          }
        }));
  }

}
