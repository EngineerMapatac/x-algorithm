package com.twitter.botmaker.function;

import java.util.Set;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableSet;
import com.google.common.collect.Sets;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;


public class DoSequential extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.<Type>of(),
      TP,
      Type.setOf(TP)
  );

  public DoSequential(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
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

    if (isFunctor(childExtractors)) {

      return new Functor<Runtime>(this, childExtractors) {

        @Override
        public Future run(Context<Runtime> context) {
          return DoSequential.this.run(context, childExtractors);
        }

        @Override
        public Object apply(Context<Runtime> context) throws Exception {
          ImmutableSet.Builder<Object> builder = ImmutableSet.builder();
          for (final Extractor extractor : childExtractors) {
            Functor functor = (Functor) extractor;
            builder.add(functor.apply(context));
          }
          return builder.build();
        }
      };
    } else {

      return new Extractor<Runtime>(this, childExtractors) {
        @Override
        public Future run(Context<Runtime> context) {
          return DoSequential.this.run(context, childExtractors);
        }
      };
    }
  }

  private Future run(Context<Runtime> context, ImmutableList<Extractor> childExtractors) {
    Future<Set<Object>> result = Future.value(Sets.newHashSet());
    for (final Extractor extractor : childExtractors) {
      result = result.flatMap(Function.func(set ->
          context.run(extractor).map(Function.func(r ->
              Sets.union(set, ImmutableSet.of(r))))));
    }
    return (Future) result;
  }

}

