package com.twitter.botmaker.function;

import scala.runtime.BoxedUnit;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public class DoAsync extends ASTNode<Runtime> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(),
      Type.OBJECT,
      Type.UNIT
  );

  public DoAsync(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  public Extractor<Runtime> toExtractor() {

    ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();

    return new Functor<Runtime>(this, childExtractors) {
      @Override
      public Future<Object> run(Context<Runtime> context) {
        return Future.apply(Function.exfunc0(() -> apply(context)));
      }

      @Override
      public Object apply(Context<Runtime> context) {
        for (Extractor extractor : childExtractors) {
          context.run(extractor).onFailure(Function.func((Throwable t) -> {
            context.reportFailure(this.getExprText(), extractor.getExprText(), t);
            return BoxedUnit.UNIT;
          }));
        }
        return unit();
      }
    };
  }
}

