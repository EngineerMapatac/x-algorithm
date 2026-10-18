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

public abstract class FunctionNode1O3<E, T1, O1, O2, O3>
    extends ASTNode<E> {

  public FunctionNode1O3(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
    assertChildrenSize(exprText, children, 1, 4);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new FunctionExtractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            return Future.value(FunctionNode1O3.this.evaluate(context, args));
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }

        @Override
        public Object apply(Context<E> context) throws Exception {
          ImmutableList<Functor> childFunctors = (ImmutableList) childExtractors;
          List<Object> args = Lists.newLinkedList();
          for (int i = 0; i < childExtractors.size(); i++) {
            args.add(childFunctors.get(i).apply(context));
          }
          return FunctionNode1O3.this.evaluate(context, args);
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            return Future.value(FunctionNode1O3.this.evaluate(context, args));
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  private Object evaluate(Context<E> context, List<Object> args) throws Exception {
    try {
      if (args.size() == 1) {
        Object result = this.apply(
            context, (T1) args.get(0));
        return result;
      } else if (args.size() == 2) {
        Object result = FunctionNode1O3.this.apply(
            context, (T1) args.get(0), (O1) args.get(1));
        return result;
      } else if (args.size() == 3) {
        Object result = FunctionNode1O3.this.apply(
            context, (T1) args.get(0), (O1) args.get(1), (O2) args.get(2));
        return result;
      } else {
        Object result = FunctionNode1O3.this.apply(
            context, (T1) args.get(0), (O1) args.get(1), (O2) args.get(2), (O3) args.get(3));
        return result;
      }
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected Object apply(Extractor<E> extractor, Context<E> context, T1 arg1,
                         O1 opt1, O2 opt2, O3 opt3) throws Exception {
    return apply(context, arg1, opt1, opt2, opt3);
  }

  protected abstract Object apply(Context<E> context, T1 arg1,
                                  O1 opt1, O2 opt2, O3 opt3) throws Exception;

  protected Object apply(Extractor<E> extractor, Context<E> context, T1 arg1, O1 opt1, O2 opt2
  ) throws Exception {
    return apply(context, arg1, opt1, opt2);
  }

  protected abstract Object apply(Context<E> context, T1 arg1, O1 opt1, O2 opt2) throws Exception;

  protected Object apply(Extractor<E> extractor, Context<E> context, T1 arg1,
                         O1 opt1) throws Exception {
    return apply(context, arg1, opt1);
  }

  protected abstract Object apply(Context<E> context, T1 arg1, O1 opt1) throws Exception;

  protected Object apply(Extractor<E> extractor, Context<E> context, T1 arg1) throws Exception {
    return apply(context, arg1);
  }

  protected abstract Object apply(Context<E> context, T1 arg1) throws Exception;

}
