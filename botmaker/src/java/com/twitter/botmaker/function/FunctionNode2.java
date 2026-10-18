package com.twitter.botmaker.function;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.util.Future;

public abstract class FunctionNode2<E, T1, T2> extends ASTNode<E> {

  public FunctionNode2(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
    assertChildrenSize(exprText, children, 2);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new Extractor.FunctionExtractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode2.this.apply(
                this, context, (T1) args.get(0), (T2) args.get(1));
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<E> context) throws Exception {
          ImmutableList<Functor> childFunctors = (ImmutableList) childExtractors;
          T1 arg1 = (T1) childFunctors.get(0).apply(context);
          T2 arg2 = (T2) childFunctors.get(1).apply(context);
          return FunctionNode2.this.apply(this, context, arg1, arg2);
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode2.this.apply(
                this, context, (T1) args.get(0), (T2) args.get(1));
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2) throws Exception {
    try {
      return apply(context, arg1, arg2);
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2) throws Exception;
}
