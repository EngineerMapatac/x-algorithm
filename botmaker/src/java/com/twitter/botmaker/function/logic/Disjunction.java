package com.twitter.botmaker.function.logic;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.ContextCancelledException;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public class Disjunction extends ASTNode<Runtime> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.BOOLEAN, Type.BOOLEAN),
      Type.BOOLEAN
  );

  public Disjunction(String exprText, ASTNode arg1, ASTNode arg2) throws SemanticCheckFailure {
    super(exprText, ImmutableList.of(arg1, arg2));
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  private static class DisjunctionExtractor extends Extractor<Runtime> {
    private final ImmutableList<Extractor> operands;

    public DisjunctionExtractor(Disjunction astNode, ImmutableList<Extractor> operands) {

      super(astNode, operands);
      this.operands = operands;
    }

    @Override
    public Future run(Context<Runtime> context) {
      return Disjunction.runRecur(context, getAstNode(), operands);
    }
  }

  private static class DisjunctionFunctor extends Extractor.Functor<Runtime> {
    private final ImmutableList<Extractor> operands;

    public DisjunctionFunctor(Disjunction astNode, ImmutableList<Extractor> operands) {

      super(astNode, operands);
      this.operands = operands;
    }


    @Override
    public Future run(Context<Runtime> context) {
      return Disjunction.runRecur(context, getAstNode(), operands);
    }

    @Override
    public Object apply(Context<Runtime> context) throws Exception {
      try {
        for (Extractor extractor : operands) {
          if (context.isCancelled()) {
            throw new ContextCancelledException();
          }

          Functor functor = (Functor) extractor;
          if ((Boolean) functor.apply(context)) {
            return true;
          }
        }

        return false;
      } catch (Exception e) {
        throw FunctionFailure.wrap(e, getAstNode(), context);
      }
    }
  }

  @Override
  public Extractor<Runtime> toExtractor() {
    ImmutableList<Extractor> childObjectExtractors = buildExtractorsOfChildren();
    Extractor argA = childObjectExtractors.get(0);
    Extractor argB = childObjectExtractors.get(1);

    ImmutableList<Extractor> operands;
    if (argA instanceof DisjunctionExtractor) {
      operands = new ImmutableList.Builder<Extractor>()
          .addAll(((DisjunctionExtractor) argA).operands)
          .add(argB)
          .build();
    } else if (argA instanceof DisjunctionFunctor) {
      operands = new ImmutableList.Builder<Extractor>()
          .addAll(((DisjunctionFunctor) argA).operands)
          .add(argB)
          .build();
    } else {
      operands = ImmutableList.of(argA, argB);
    }

    if (isFunctor(operands)) {
      return new DisjunctionFunctor(this, operands);
    } else {
      return new DisjunctionExtractor(this, operands);
    }
  }

  private static Future<Object> runRecur(
      Context<Runtime> context, ASTNode<?> astNode, List<Extractor> operands) {
    return runRecur(context, astNode, operands, 0)
        .rescue(Function.func((Throwable ex) ->
            Future.exception(FunctionFailure.wrap(ex, astNode, context))));
  }

  private static Future<Object> runRecur(
      Context<Runtime> context, ASTNode<?> astNode, List<Extractor> operands, int idx) {

    try {
      int current = idx;
      while (current < operands.size()) {
        Extractor extractor = operands.get(current);
        if (context.isCancelled()) {
          return Future.exception(new ContextCancelledException());
        } else if (!context.isDebugging() && extractor instanceof Extractor.Functor) {
          Extractor.Functor functor = (Extractor.Functor) extractor;
          boolean result = (Boolean) functor.apply(context);
          if (result) {
            return Future.value(result);
          }
          current++;
        } else {
          Future<Object> futureResult = context.run(extractor);
          if (current == operands.size() - 1) {
            return futureResult;
          }
          int next = current + 1;
          return futureResult.flatMap(val -> {
            if ((Boolean) val) {
              return Future.value(true);
            } else {
              return runRecur(context, astNode, operands, next);
            }
          });
        }
      }
      return Future.value(false);
    } catch (Exception e) {
      return Future.exception(FunctionFailure.wrap(e, astNode, context));
    }
  }
}
