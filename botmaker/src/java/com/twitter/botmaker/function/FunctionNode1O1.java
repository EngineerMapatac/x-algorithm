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

public abstract class FunctionNode1O1<E, T1, O1> extends ASTNode<E> {

  public FunctionNode1O1(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
    assertChildrenSize(exprText, children, 1, 2);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new FunctionExtractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            if (childExtractors.size() == 1) {
              Object result = FunctionNode1O1.this.apply(
                  this, context, (T1) args.get(0));
              return Future.value(result);
            } else {
              Object result = FunctionNode1O1.this.apply(
                  this, context, (T1) args.get(0), (O1) args.get(1));
              return Future.value(result);
            }
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<E> context) throws Exception {
          ImmutableList<Functor> childFunctors = (ImmutableList) childExtractors;
          T1 arg1 = (T1) childFunctors.get(0).apply(context);
          if (childExtractors.size() == 1) {
            return FunctionNode1O1.this.apply(this, context, arg1);
          } else {
            O1 opt1 = (O1) childFunctors.get(1).apply(context);
            return FunctionNode1O1.this.apply(this, context, arg1, opt1);
          }
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            if (childExtractors.size() == 1) {
              Object result = FunctionNode1O1.this.apply(
                  this, context, (T1) args.get(0));
              return Future.value(result);
            } else {
              Object result = FunctionNode1O1.this.apply(
                  this, context, (T1) args.get(0), (O1) args.get(1));
              return Future.value(result);
            }
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, O1 opt1) throws Exception {
    try {
      return apply(context, arg1, opt1);
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }

  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, O1 opt1) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1) throws Exception {
    try {
      return apply(context, arg1);
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1) throws Exception;

}
