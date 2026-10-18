package com.twitter.botmaker;

import java.util.LinkedList;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentMap;
import java.util.function.BiFunction;
import java.util.function.BooleanSupplier;
import java.util.function.Supplier;

import scala.Function0;
import scala.PartialFunction;
import scala.runtime.BoxedUnit;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.exceptions.MissingFeatureException;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.StackFrame;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.util.TriConsumer;
import com.twitter.util.Future;

public abstract class Context<E> {

  private static final SemanticCheckFailure VARIABLE_NOT_ALLOWED =
      new SemanticCheckFailure("Variables not allowed in root context");

  private static final StackFrame MAIN_STACK_FRAME = new StackFrame("main", -1);

  protected static final Supplier<List<StackFrame>> DEFAULT_STACK_FRAMES =
      () -> {
        LinkedList<StackFrame> list = new LinkedList<>();
        list.add(MAIN_STACK_FRAME);
        return list;
      };

  public final E getRuntime() {
    return runtimeSupplier().get();
  }

  public abstract Supplier<E> runtimeSupplier();

  public abstract long getTrackingId();

  public abstract long nowMillis();

  public abstract long startMillis();

  public Context<E> getRootContext() {
    return this;
  }

  public final boolean isCancelled() {
    return cancelledSupplier().getAsBoolean();
  }

  public BooleanSupplier cancelledSupplier() {
    return () -> false;
  }

  public final boolean isDebugging() {
    return debugSupplier().getAsBoolean();
  }

  public BooleanSupplier debugSupplier() {
    return () -> false;
  }

  public final Future<Object> run(Extractor<E> extractor) {
    return extractorEvaluationFunction().apply(this, extractor);
  }

  protected abstract BiFunction<Context<E>, Extractor<E>, Future<Object>>
  extractorEvaluationFunction();

  public final boolean isInputFeatureDefined(String featureName) {
    return inputFeatureFunction().isDefinedAt(featureName);
  }

  public final Object getInputFeature(String featureName) throws MissingFeatureException {
    return inputFeatureFunction().applyOrElse(featureName, (String fn) -> {
      throw new MissingFeatureException(fn);
    });
  }

  public abstract PartialFunction<String, Object> inputFeatureFunction();

  public Future<Object> getVariable(String variableName) {
    return Future.exception(VARIABLE_NOT_ALLOWED);
  }

  public void newVariable(String name, Future<Object> value) throws SemanticCheckFailure {
    throw VARIABLE_NOT_ALLOWED;
  }

  public abstract List<StackFrame> getStackFrames();

  public abstract void addOutputFeature(Object namespace, String name, Type type, Object value);

  public abstract void reportStat(Object namespace, String statName, long value);

  public abstract void reportFailure(Object namespace, String category, Throwable t);

  public abstract TriConsumer<String, ActionLevel, List<?>> reportFunctionEvaluationConsumer();

  public abstract <T> Future<T> applyActionRateLimit(
      Context<E> context,
      String functionName,
      ActionLevel actionLevel,
      Function0<Future<T>> func,
      List<?> args
  );

  public final void reportFunctionEvaluation(String functionName,
                                             ActionLevel actionLevel,
                                             List<?> args) {
    reportFunctionEvaluationConsumer().accept(functionName, actionLevel, args);
  }

  public static ContextBuilder builder() {
    return new ContextBuilder();
  }

  public static class Proxy<T> extends Context<T> {

    protected final Context<T> proxied;

    private final BooleanSupplier cancelSupplier;
    private final BooleanSupplier debugSupplier;
    private final TriConsumer<String, ActionLevel, List<?>> reportFunctionEvaluationConsumer;
    private final Supplier<T> runtimeSupplier;
    private final PartialFunction<String, Object> inputFeatureFunction;
    private final BiFunction<Context<T>, Extractor<T>, Future<Object>> extractorEvaluationFunction;

    public Proxy(Context<T> proxied) {
      this.proxied = proxied;
      this.cancelSupplier = proxied.cancelledSupplier();
      this.debugSupplier = proxied.debugSupplier();
      this.reportFunctionEvaluationConsumer = proxied.reportFunctionEvaluationConsumer();
      this.runtimeSupplier = proxied.runtimeSupplier();
      this.inputFeatureFunction = proxied.inputFeatureFunction();
      this.extractorEvaluationFunction = proxied.extractorEvaluationFunction();
    }

    public Supplier<T> runtimeSupplier() {
      return runtimeSupplier;
    }

    @Override
    public Context<T> getRootContext() {
      return proxied.getRootContext();
    }

    @Override
    public long getTrackingId() {
      return proxied.getTrackingId();
    }

    @Override
    public long nowMillis() {
      return proxied.nowMillis();
    }

    @Override
    public long startMillis() {
      return proxied.startMillis();
    }

    @Override
    public BooleanSupplier cancelledSupplier() {
      return cancelSupplier;
    }

    @Override
    public BooleanSupplier debugSupplier() {
      return debugSupplier;
    }

    @Override
    public BiFunction<Context<T>, Extractor<T>, Future<Object>> extractorEvaluationFunction() {
      return extractorEvaluationFunction;
    }

    @Override
    public PartialFunction<String, Object> inputFeatureFunction() {
      return inputFeatureFunction;
    }

    @Override
    public Future<Object> getVariable(String variableName) {
      return proxied.getVariable(variableName);
    }

    protected Future<Object> getVariable(
        String variableName,
        ConcurrentMap<String, Future<Object>> scopeVariables
    ) {
      if (scopeVariables.containsKey(variableName)) {
        return scopeVariables.get(variableName);
      } else {
        Future value = this.proxied.getVariable(variableName);
        scopeVariables.put(variableName, value);
        return value;
      }
    }

    @Override
    public void newVariable(String name, Future<Object> value) throws SemanticCheckFailure {
      proxied.newVariable(name, value);
    }

    protected static void newVariable(
        String name,
        Future<Object> value,
        ConcurrentMap<String, Future<Object>> scopeVariables
    ) {
      scopeVariables.put(name, value);
    }

    @Override
    public List<StackFrame> getStackFrames() {
      return proxied.getStackFrames();
    }

    protected List<StackFrame> getStackFrames(StackFrame stackFrame) {
      List<StackFrame> list = proxied.getStackFrames();
      list.add(0, stackFrame);
      return list;
    }

    @Override
    public void addOutputFeature(Object namespace, String name, Type type, Object value) {
      proxied.addOutputFeature(namespace, name, type, value);
    }

    @Override
    public void reportStat(Object namespace, String statName, long value) {
      proxied.reportStat(namespace, statName, value);
    }

    @Override
    public void reportFailure(Object namespace, String category, Throwable t) {
      proxied.reportFailure(namespace, category, t);
    }

    @Override
    public TriConsumer<String, ActionLevel, List<?>> reportFunctionEvaluationConsumer() {
      return reportFunctionEvaluationConsumer;
    }

    @Override
    public <V> Future<V> applyActionRateLimit(
        Context<T> context, String functionName, ActionLevel actionLevel, Function0<Future<V>> func,
        List<?> args
    ) {
      return proxied.applyActionRateLimit(
          context,
          functionName,
          actionLevel,
          func,
          ImmutableList.of()
      );
    }
  }

  public static <T> Context<T> of(
      Context<T> proxied,
      ConcurrentMap<String, Future<Object>> scopeVariables
  ) {
    return new Proxy<T>(proxied) {
      @Override
      public Future<Object> getVariable(String variableName) {
        return getVariable(variableName, scopeVariables);
      }

      @Override
      public void newVariable(String name, Future<Object> value) {
        newVariable(name, value, scopeVariables);
      }
    };
  }

  public static <T> Context<T> of(
      Context<T> proxied,
      StackFrame stackFrame
  ) {
    return new Proxy<T>(proxied) {
      @Override
      public List<StackFrame> getStackFrames() {
        return getStackFrames(stackFrame);
      }
    };
  }

  public static <T> Context<T> of(
      Context<T> proxied,
      ConcurrentMap<String, Future<Object>> scopeVariables,
      StackFrame stackFrame
  ) {
    return new Proxy<T>(proxied) {
      @Override
      public Future<Object> getVariable(String variableName) {
        return getVariable(variableName, scopeVariables);
      }

      @Override
      public void newVariable(String name, Future<Object> value) {
        newVariable(name, value, scopeVariables);
      }

      @Override
      public List<StackFrame> getStackFrames() {
        return getStackFrames(stackFrame);
      }
    };
  }

  public static <T> Context<T> ofVariableUsageTracking(
      Context<T> proxied,
      ConcurrentMap<String, Future<Object>> scopeVariables,
      Set<String> usedVariables) {
    return new Proxy<T>(proxied) {
      @Override
      public Future<Object> getVariable(String variableName) {
        usedVariables.add(variableName);
        return getVariable(variableName, scopeVariables);
      }

      @Override
      public void newVariable(String name, Future<Object> value) {
        newVariable(name, value, scopeVariables);
      }
    };
  }

  public interface TraceCollector {

    boolean filter(Extractor extractor);

    void info(
        long trackingId,
        Extractor extractor,
        Object resultOrException,
        long startMs,
        long endMs);
  }

  public static <T> Context<T> trace(Context<T> proxied, TraceCollector collector) {
    return new Proxy<T>(proxied) {

      @Override
      public BooleanSupplier debugSupplier() {
        return () -> true;
      }

      @Override
      public BiFunction<Context<T>, Extractor<T>, Future<Object>> extractorEvaluationFunction() {
        return (ctx, extractor) -> {
          long startMs = nowMillis();
          Future<Object> resultFut = proxied.extractorEvaluationFunction().apply(ctx, extractor);
          if (!collector.filter(extractor)) {
            return resultFut;
          }
          return resultFut.onSuccess(result -> {
            collector.info(getTrackingId(), extractor, result, startMs, nowMillis());
            return BoxedUnit.UNIT;
          }).onFailure(t -> {
            collector.info(getTrackingId(), extractor, t, startMs, nowMillis());
            return BoxedUnit.UNIT;
          });
        };
      }

    };
  }
}
