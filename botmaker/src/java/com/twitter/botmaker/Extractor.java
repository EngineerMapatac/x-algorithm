package com.twitter.botmaker;

import java.util.List;
import java.util.Objects;

import scala.PartialFunction;
import scala.Unit;
import scala.runtime.BoxedUnit;

import com.google.common.base.Optional;
import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;

import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.util.Duration;
import com.twitter.util.Function;
import com.twitter.util.Future;

public abstract class Extractor<E> {

  private final ASTNode astNode;
  private final ImmutableList<Extractor> childrenExtractors;

  private Boolean isCacheable;
  private Optional<Duration> timeoutDuration;

  public Extractor(ASTNode astNode, ImmutableList<Extractor> children) {
    this.astNode = astNode;
    this.childrenExtractors = children;
    this.timeoutDuration = Optional.absent();
  }

  public final ASTNode getAstNode() {
    return astNode;
  }

  public final String getClassName() {
    return astNode.getClassName();
  }

  public final String getExprText() {
    return astNode.getExprText();
  }

  public final ImmutableList<Extractor> getChildren() {
    return childrenExtractors;
  }

  public final Fingerprint getFingerprint() {
    return astNode.getFingerprint();
  }

  public final boolean isCacheable() {
    return isCacheable != null ? isCacheable : false;
  }

  public final void setCacheable(boolean cacheable) {
    Preconditions.checkState(isCacheable == null
            || isCacheable == cacheable,
        "A conflict with preset extractor cacheability.");
    this.isCacheable = cacheable;
  }

  public final Optional<Duration> getTimeoutDuration() {
    return timeoutDuration;
  }

  public final void setTimeoutDuration(Duration timeout) {
    timeoutDuration = Optional.of(timeout);
  }

  @Override
  public boolean equals(Object object) {
    if (this == object) {
      return true;
    }
    if (!(object instanceof Extractor)) {
      return false;
    }
    Extractor that = (Extractor) object;
    return Objects.equals(this.getFingerprint(), that.getFingerprint())
        && Objects.equals(this.isCacheable, that.isCacheable)
        && Objects.equals(this.timeoutDuration, that.timeoutDuration);
  }

  @Override
  public int hashCode() {
    return Objects.hash(getFingerprint(), isCacheable, timeoutDuration);
  }

  public abstract Future<Object> run(Context<E> context);

  public final void acceptDefinedVisitor(PartialFunction<Extractor, Unit> operation) {
    if (operation.isDefinedAt(this)) {
      operation.apply(this);

      if (this.getChildren() != null) {
        for (Extractor child : this.getChildren()) {
          child.acceptDefinedVisitor(operation);
        }
      }
    }
  }

  protected final BoxedUnit unit() {
    return BoxedUnit.UNIT;
  }

  public abstract static class GeneralExractor<T> extends Extractor<T> {
    public GeneralExractor(ASTNode astNode, ImmutableList<Extractor> children) {
      super(astNode, children);
    }

    @Override
    public Future<Object> run(Context<T> context) {
      return evaluateChildren(context).flatMap(
          Function.func(args -> evaluate(context, args)));
    }

    @SuppressWarnings("unchecked")
    public Future<List<Object>> evaluateChildren(Context<T> context) {
      ImmutableList<Extractor> childExtractors = getChildren();
      List<Future<Object>> childFutures = Lists.newArrayListWithCapacity(childExtractors.size());

      for (Extractor extractor : childExtractors) {
        Future arg = context.run(extractor);
        childFutures.add(arg);
      }

      return Future.collect(childFutures);
    }

    public abstract Future<Object> evaluate(Context<T> context, List<Object> args);
  }

  public abstract static class Functor<T> extends Extractor<T> {

    public Functor(ASTNode astNode, ImmutableList<Extractor> children) {
      super(astNode, children);
    }

    public abstract Object apply(Context<T> context) throws Exception;
  }

  public abstract static class FunctionExtractor<T> extends Functor<T> {

    public FunctionExtractor(ASTNode astNode, ImmutableList<Extractor> children) {
      super(astNode, children);
    }

    @Override
    public Future<Object> run(Context<T> context) {
      return evaluateChildren(context).flatMap(
          Function.func(args -> evaluate(context, args)));
    }

    @SuppressWarnings("unchecked")
    public Future<List<Object>> evaluateChildren(Context<T> context) {
      ImmutableList<Extractor> childExtractors = getChildren();
      List<Future<Object>> childFutures = Lists.newArrayListWithCapacity(childExtractors.size());

      for (Extractor extractor : childExtractors) {
        Future arg = context.run(extractor);
        childFutures.add(arg);
      }

      return Future.collect(childFutures);
    }

    public abstract Future<Object> evaluate(Context<T> context, List<Object> args);
  }

}
