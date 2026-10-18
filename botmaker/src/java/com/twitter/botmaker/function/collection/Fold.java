package com.twitter.botmaker.function.collection;

import java.util.Collection;
import java.util.Iterator;
import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public final class Fold {

  private static final Type TP1 = Type.newGenericType();
  private static final ASTNode.Signature FOLDL_SIGNATURE = new ASTNode.Signature(
      ImmutableList.of(TP1, Type.OBJECT, Type.collectionOf(Type.OBJECT)),
      TP1
  );

  private static final Type TP2 = Type.newGenericType();
  private static final ASTNode.Signature FOLDL1_SIGNATURE = new ASTNode.Signature(
      ImmutableList.of(TP2, Type.collectionOf(Type.OBJECT)),
      TP2
  );

  private Fold() {
  }

  private static final class FoldLeftFunction {
    private final String varA;
    private final String varB;
    private final Iterator<Object> iterator;
    private final Extractor funcExtractor;
    private final Context<Runtime> context;

    private FoldLeftFunction(
        String varA, String varB, Iterator<Object> iterator, Extractor funcExtractor,
        Context<Runtime> context) {
      this.varA = varA;
      this.varB = varB;
      this.iterator = iterator;
      this.funcExtractor = funcExtractor;
      this.context = context;
    }

    private Future<Object> runFoldLeft(Future<Object> beginValueFuture) {
      if (!iterator.hasNext()) {
        return beginValueFuture;
      }
      final Object nextValue = iterator.next();
      return beginValueFuture.flatMap(
          new Function<Object, Future<Object>>() {
            @Override
            public Future<Object> apply(Object beginValue) {
              ConcurrentMap<String, Future<Object>> variableMap = Maps.newConcurrentMap();
              variableMap.put(varA, Future.value(beginValue));
              variableMap.put(varB, Future.value(nextValue));
              Context newContext = Context.of(context, variableMap);
              Future<Object> newBeginValue = newContext.run(funcExtractor);
              return runFoldLeft(newBeginValue);
            }
          }
      );
    }

    public static Future evaluate(
        Context<Runtime> context, Future<Object> beginValueFuture,
        String varA, String varB, Iterator<Object> iterator, Extractor funcExtractor) {
      return new FoldLeftFunction(
          varA, varB, iterator, funcExtractor, context
      ).runFoldLeft(beginValueFuture);
    }
  }

  public static ASTNode createFoldl(
      String exprText,
      final String varA,
      final String varB,
      ASTNode funcNode,
      ASTNode seedValueNode,
      long scope,
      ASTNode collectionNode)
      throws SemanticCheckFailure {

    return new ASTNode<Runtime>(
        exprText, ImmutableList.of(funcNode, seedValueNode, collectionNode)) {
      @Override
      public long computeFingerprintScope(long currentScope) {
        return super.computeFingerprintScope(Math.min(currentScope, scope));
      }

      @Override
      public Signature getSignature() {
        return FOLDL_SIGNATURE;
      }

      @Override
      protected CacheLevel getCacheLevel() {
        return CacheLevel.Global;
      }

      @Override
      public Extractor<Runtime> toExtractor() {
        Extractor funcExtractor = funcNode.toExtractor();
        Extractor seedValueExtractor = seedValueNode.toExtractor();
        Extractor collectionExtractor = collectionNode.toExtractor();
        ImmutableList<Extractor> children =
            ImmutableList.of(funcExtractor, seedValueExtractor, collectionExtractor);

        return new Extractor<Runtime>(this, children) {

          @Override
          public Future run(Context<Runtime> context) {
            return context.run(collectionExtractor).flatMap((Object collectionObj) -> {
              @SuppressWarnings("unchecked")
              Collection<Object> collection = (Collection<Object>) collectionObj;
              return FoldLeftFunction.evaluate(
                  context,
                  context.run(seedValueExtractor),
                  varA, varB, collection.iterator(), funcExtractor
              );
            }).rescue(Function.func((Throwable ex) ->
                Future.exception(FunctionFailure.wrap(ex, getAstNode(), context))));
          }
        };
      }
    };
  }

  public static ASTNode createFoldl1(
      String exprText,
      final String varA,
      final String varB,
      ASTNode funcNode,
      long scope,
      ASTNode collectionNode) throws SemanticCheckFailure {

    return new ASTNode<Runtime>(exprText, ImmutableList.of(funcNode, collectionNode)) {
      @Override
      public Signature getSignature() {
        return FOLDL1_SIGNATURE;
      }

      @Override
      protected CacheLevel getCacheLevel() {
        return CacheLevel.Global;
      }

      @Override
      public long computeFingerprintScope(long currentScope) {
        return super.computeFingerprintScope(Math.min(currentScope, scope));
      }

      @Override
      public Extractor<Runtime> toExtractor() {

        Extractor funcExtractor = funcNode.toExtractor();
        Extractor collectionExtractor = collectionNode.toExtractor();
        ImmutableList<Extractor> children = ImmutableList.of(funcExtractor, collectionExtractor);

        return new Extractor<Runtime>(this, children) {

          @Override
          public Future run(Context<Runtime> context) {
            return context.run(collectionExtractor).flatMap((Object collectionObj) -> {
              @SuppressWarnings("unchecked")
              Collection<Object> collection = (Collection<Object>) collectionObj;
              Iterator<Object> iterator = collection.iterator();
              if (!iterator.hasNext()) {
                throw new RuntimeException("cannot pass empty collection to Foldl1()");
              }
              return FoldLeftFunction.evaluate(
                  context, Future.value(iterator.next()),
                  varA, varB, iterator, funcExtractor);
            }).rescue(Function.func((Throwable ex) ->
                    Future.exception(FunctionFailure.wrap(ex, getAstNode(), context))));
          }
        };
      }
    };
  }
}
