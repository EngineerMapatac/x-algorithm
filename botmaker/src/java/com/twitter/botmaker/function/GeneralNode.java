package com.twitter.botmaker.function;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.util.Function;
import com.twitter.util.Future;

public abstract class GeneralNode<E> extends ASTNode<E> {
  public GeneralNode(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public final Extractor<E> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    return new Extractor.GeneralExractor<E>(this, childExtractors) {

      @Override
      public Future<Object> evaluate(Context<E> context, List<Object> args) {
        try {
          return GeneralNode.this.evaluate(this, context, args).rescue(
              new Function<Throwable, Future<Object>>() {
                @Override
                public boolean isDefinedAt(Throwable t) {
                  return !(t instanceof FunctionFailure);
                }

                @Override
                public Future<Object> apply(Throwable t) {
                  return Future.exception(
                      new FunctionFailure(getAstNode(), context.getStackFrames(), t));
                }
              }
          );
        } catch (FunctionFailure ff) {
          throw ff;
        } catch (Exception t) {
          throw new FunctionFailure(getAstNode(), context.getStackFrames(), t);
        }
      }
    };
  }

  protected Future<Object> evaluate(
      Extractor<E> extractor, Context<E> context, List<Object> args) {
    return evaluate(context, args);
  }

  protected abstract Future<Object> evaluate(Context<E> context, List<Object> args);

  protected <T> T getOptionalParamOrDefault(List<Object> args, int index, T defaultValue) {
    if (args.size() > index) {
      return (T) args.get(index);
    } else {
      return defaultValue;
    }
  }
}
