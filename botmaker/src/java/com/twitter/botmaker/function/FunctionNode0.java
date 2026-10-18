package com.twitter.botmaker.function;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.FunctionExtractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.util.Future;

public abstract class FunctionNode0<E> extends ASTNode<E> {

  public FunctionNode0(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
    assertChildrenSize(exprText, children, 0);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new FunctionExtractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode0.this.apply(this, context);
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<E> context) throws Exception {
          return FunctionNode0.this.apply(this, context);
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            Object result = FunctionNode0.this.apply(this, context);
            return Future.value(result);
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  protected Object apply(Extractor<E> extractor, Context<E> context) throws Exception {
    try {
      return apply(context);
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected abstract Object apply(
      Context<E> context) throws Exception;
}
