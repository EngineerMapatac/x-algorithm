package com.twitter.botmaker.function;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;
import com.google.common.collect.Sets;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.ContextCancelledException;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Future;
import com.twitter.util.Futures;

public class Block extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(TP),
      Type.OBJECT,
      TP);

  private final ImmutableList<Pair<String, Type>> variables;

  public Block(
      String exprText, ImmutableList<ASTNode> children,
      ImmutableList<Pair<String, Type>> variables) throws SemanticCheckFailure {
    super(exprText, ImmutableList.copyOf(Lists.reverse(children)));
    this.variables = internVariableNames(variables);
  }

  private static <T> ImmutableList<Pair<String, T>> internVariableNames(
      List<Pair<String, T>> variableList) {
    List<Pair<String, T>> accumulation = Lists.newArrayListWithCapacity(variableList.size());
    for (Pair<String, T> oldPair : variableList) {
      accumulation.add(new Pair<>(intern(oldPair.getFirst()), oldPair.getSecond()));
    }
    return ImmutableList.copyOf(accumulation);
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
  public Extractor<Runtime> toExtractor() {
    ImmutableList<Extractor> children = buildExtractorsOfChildren().reverse();

    if (variables.size() == 0 && isFunctor(children)) {
      return new Functor<Runtime>(this, children) {

        @Override
        public Future run(Context<Runtime> context) {
          try {
            return Future.value(apply(context));
          } catch (Exception e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<Runtime> context) throws Exception {
          Object result = unit();
          for (Extractor extractor : children) {
            if (context.isCancelled()) {
              throw new ContextCancelledException();
            }

            Functor functor = (Functor) extractor;
            result = functor.apply(context);
          }
          return result;
        }
      };
    } else if (variables.size() == 0) {
      return new Extractor<Runtime>(this, children) {
        @Override
        public Future<Object> run(Context<Runtime> context) {
          return Block.this.runChildStatements(context, children, 0);
        }
      };
    } else {
      return new Extractor<Runtime>(this, children) {
        @Override
        public Future run(Context<Runtime> context) {
          ConcurrentMap<String, Future<Object>> variableValues = Maps.newConcurrentMap();
          Set<String> usedVariables = Sets.newConcurrentHashSet();
          Context<Runtime> newContext =
              Context.ofVariableUsageTracking(context, variableValues, usedVariables);
          return Block.this.runChildStatements(
              newContext,
              children,
              0).flatMap((Object result) -> {
            List<Future<Object>> outstandingVariableFutures = new ArrayList<>();
            for (String key : variableValues.keySet()) {
              if (!usedVariables.contains(key)) {
                outstandingVariableFutures.add(variableValues.get(key));
              }
            }
            return Futures.join(outstandingVariableFutures).map(ignored -> result);
          });
        }
      };
    }
  }

  @SuppressWarnings("unchecked")
  private Future<Object> runChildStatements(
      Context<Runtime> context, ImmutableList<Extractor> children, int fromChildIndex) {

    if (fromChildIndex < 0 || fromChildIndex >= children.size()) {
      return Future.exception(new IllegalStateException(
          String.format("Cannot run statement %d of the block", fromChildIndex)));
    }

    try {
      int current = fromChildIndex;
      Object result = unit();
      while (current < children.size()) {
        Extractor currentExtractor = children.get(current);
        if (context.isCancelled()) {
          return Future.exception(new ContextCancelledException());
        } else if (currentExtractor instanceof Functor && !context.isDebugging()) {
          Functor functor = (Functor) currentExtractor;
          result = functor.apply(context);
          current++;
        } else {
          int nextIndex = current + 1;
          Future<Object> valueFuture = context.run(currentExtractor);
          if (nextIndex < children.size()) {
            return valueFuture.flatMap(ignored -> runChildStatements(context, children, nextIndex));
          }
          return valueFuture;
        }
      }
      return Future.value(result);
    } catch (Exception e) {
      return Future.exception(e);
    }
  }
}
