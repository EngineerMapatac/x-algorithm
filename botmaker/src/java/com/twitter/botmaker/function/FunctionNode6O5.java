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

public abstract class FunctionNode6O5<E, T1, T2, T3, T4, T5, T6, O1, O2, O3, O4, O5>
    extends ASTNode<E> {

  public FunctionNode6O5(
      String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
    assertChildrenSize(exprText, children, 6, 11);
  }

  @Override
  public Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    if (isFunctor(childExtractors)) {
      return new FunctionExtractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            return Future.value(FunctionNode6O5.this.evaluate(context, args));
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
          return FunctionNode6O5.this.evaluate(context, args);
        }
      };
    } else {
      return new Extractor.GeneralExractor<E>(this, childExtractors) {
        @Override
        public Future<Object> evaluate(Context<E> context, List<Object> args) {
          try {
            return Future.value(FunctionNode6O5.this.evaluate(context, args));
          } catch (Throwable e) {
            return Future.exception(e);
          }
        }
      };
    }
  }

  private Object evaluate(Context<E> context, List<Object> args) throws Exception {
    try {
      if (args.size() == 6) {
        Object result = this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5));
        return result;
      } else if (args.size() == 7) {
        Object result = FunctionNode6O5.this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5), (O1) args.get(6));
        return result;
      } else if (args.size() == 8) {
        Object result = FunctionNode6O5.this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5), (O1) args.get(6),
            (O2) args.get(7));
        return result;
      } else if (args.size() == 9) {
        Object result = FunctionNode6O5.this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5), (O1) args.get(6),
            (O2) args.get(7), (O3) args.get(8));
        return result;
      } else if (args.size() == 10) {
        Object result = FunctionNode6O5.this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5), (O1) args.get(6),
            (O2) args.get(7), (O3) args.get(8), (O4) args.get(9));
        return result;
      } else {
        Object result = FunctionNode6O5.this.apply(
            context, (T1) args.get(0), (T2) args.get(1), (T3) args.get(2),
            (T4) args.get(3), (T5) args.get(4), (T6) args.get(5), (O1) args.get(6),
            (O2) args.get(7), (O3) args.get(8), (O4) args.get(9), (O5) args.get(10));
        return result;
      }
    } catch (FunctionFailure ff) {
      throw ff;
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3, O4 opt4, O5 opt5) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6, opt1, opt2, opt3, opt4, opt5);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3, O4 opt4, O5 opt5) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3, O4 opt4) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6, opt1, opt2, opt3, opt4);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3, O4 opt4) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6, opt1, opt2, opt3);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2, O3 opt3) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6, opt1, opt2);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1, O2 opt2) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6, opt1);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6, O1 opt1) throws Exception;

  protected Object apply(
      Extractor<E> extractor, Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6) throws Exception {
    return apply(context, arg1, arg2, arg3, arg4, arg5, arg6);
  }

  protected abstract Object apply(
      Context<E> context, T1 arg1, T2 arg2, T3 arg3,
      T4 arg4, T5 arg5, T6 arg6) throws Exception;

}
