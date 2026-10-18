package com.twitter.botmaker.function;

import java.util.List;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.FunctionExtractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.util.Future;

public abstract class FunctionNode<E> extends ASTNode<E> {

  public FunctionNode(
      String exprText, List<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new FunctionExtractor<E>(this, childExtractors) {

        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode.this.apply(this, context, args);
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<E> context) throws Exception {
          List<Object> args = Lists.newArrayListWithCapacity(childExtractors.size());
          ImmutableList<Functor> childFunctors = (ImmutableList) childExtractors;
          for (Functor functor : childFunctors) {
            args.add(functor.apply(context));
          }

          return FunctionNode.this.apply(this, context, args);
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode.this.apply(this, context, args);
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  protected Object apply(
      Extractor<E> extractor, Context<E> context, List<Object> args) throws Exception {
    try {
      return apply(context, args);
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected abstract Object apply(Context<E> context, List<Object> args) throws Exception;
}
