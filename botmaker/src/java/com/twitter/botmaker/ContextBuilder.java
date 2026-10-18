package com.twitter.botmaker;

import java.time.Clock;
import java.util.List;
import java.util.function.BiFunction;
import java.util.function.Supplier;

import scala.Function0;
import scala.PartialFunction;
import scala.collection.JavaConverters$;

import com.google.common.collect.ImmutableMap;

import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.tuples.StackFrame;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.util.TriConsumer;
import com.twitter.util.Future;

public final class ContextBuilder {

  private final ImmutableMap.Builder<String, Object> inputFeatureBuilder = ImmutableMap.builder();
  private Clock clock = Clock.systemUTC();

  public ContextBuilder addInputFeature(String featureName, Object featureValue) {
    inputFeatureBuilder.put(featureName, featureValue);
    return this;
  }

  public ContextBuilder addClock(Clock thatClock) {
    this.clock = thatClock;
    return this;
  }

  public <E> Context<E> build(E runtime) {
    long startTimeMillis = clock.millis();
    PartialFunction<String, Object> inputFeatureFunction =
        JavaConverters$.MODULE$.mapAsScalaMap(inputFeatureBuilder.build());
    return new Context<E>() {

      @Override
      public Supplier<E> runtimeSupplier() {
        return () -> runtime;
      }

      @Override
      public long getTrackingId() {
        return 0;
      }

      @Override
      public long nowMillis() {
        return clock.millis();
      }

      @Override
      public long startMillis() {
        return startTimeMillis;
      }

      @Override
      protected BiFunction<Context<E>, Extractor<E>, Future<Object>> extractorEvaluationFunction() {
        return (ctx, extractor) -> extractor.run(ctx);
      }

      @Override
      public PartialFunction<String, Object> inputFeatureFunction() {
        return inputFeatureFunction;
      }

      @Override
      public void addOutputFeature(Object namespace, String name, Type type, Object value) {
      }

      @Override
      public void reportStat(Object namespace, String statName, long value) {
      }

      @Override
      public void reportFailure(Object namespace, String category, Throwable t) {
      }

      @Override
      public List<StackFrame> getStackFrames() {
        return Context.DEFAULT_STACK_FRAMES.get();
      }

      @Override
      public TriConsumer<String, ActionLevel, List<?>> reportFunctionEvaluationConsumer() {
        return (ignored1, ignored2, ignored3) -> {
        };
      }

      @Override
      public <T> Future<T> applyActionRateLimit(
          Context<E> context, String functionName, ActionLevel actionLevel,
          Function0<Future<T>> func,
          List<?> args
      ) {
        return func.apply();
      }
    };
  }

}


