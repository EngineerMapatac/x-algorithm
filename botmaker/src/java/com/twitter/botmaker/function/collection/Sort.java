package com.twitter.botmaker.function.collection;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Await;
import com.twitter.util.Future;

public final class Sort {

  private static final Type TP1 = Type.newGenericType();
  private static final ASTNode.Signature SIGNATURE1 = new ASTNode.Signature(
      ImmutableList.of(Type.collectionOf(TP1)),
      Type.listOf(TP1)
  );

  private static final Type TP2 = Type.newGenericType();
  private static final ASTNode.Signature SIGNATURE2 = new ASTNode.Signature(
      ImmutableList.of(Type.LONG, Type.collectionOf(TP2)),
      Type.listOf(TP2)
  );

  private Sort() {
  }

  public static ASTNode toSort(
      String exprText,
      ASTNode collectionNode) throws SemanticCheckFailure {

    final Type elementType;
    if (Collection.class.isAssignableFrom(collectionNode.getReturnType().typeBase)) {
      elementType = collectionNode.getReturnType().getTypeParams().get(0);
    } else {
      elementType = Type.OBJECT;
    }

    if (!Comparable.class.isAssignableFrom(elementType.typeBase)) {
      throw new SemanticCheckFailure(
          String.format("Sort expects a collection of %s objects, but gets %s",
              Comparable.class.getName(),
              elementType)
      );
    }

    return new FunctionNode1<Runtime, Collection<Object>>(exprText,
        ImmutableList.of(collectionNode)) {

      @Override
      public List<Comparable> apply(Context<Runtime> context, Collection<Object> collection) {
        Collection<Comparable> comparableCollection = (Collection) collection;
        List<Comparable> listToBeSorted = new ArrayList<>(comparableCollection);
        Collections.sort(listToBeSorted);
        return listToBeSorted;
      }

      @Override
      public Signature getSignature() {
        return SIGNATURE1;
      }

      @Override
      protected CacheLevel getCacheLevel() {
        return CacheLevel.Global;
      }

    };
  }

  public static ASTNode toSort(
      String exprText,
      final String varA,
      final String varB,
      ASTNode funcNode,
      long scope,
      ASTNode collectionNode) throws SemanticCheckFailure {

    return new ASTNode<Runtime>(exprText, ImmutableList.of(funcNode, collectionNode)) {
      @Override
      public Signature getSignature() {
        return SIGNATURE2;
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
            final Comparator<Object> comparator;

            if (funcExtractor instanceof Extractor.Functor) {
              Extractor.Functor functor = (Extractor.Functor) funcExtractor;
              comparator = (Object o1, Object o2) -> {
                ConcurrentMap<String, Future<Object>> variableMap = Maps.newConcurrentMap();
                variableMap.put(varA, Future.value(o1));
                variableMap.put(varB, Future.value(o2));
                Context newContext = Context.of(context, variableMap);
                try {
                  Long result = (Long) functor.apply(newContext);
                  return result.intValue();
                } catch (Exception e) {
                  throw FunctionFailure.wrap(e, getAstNode(), context);
                }
              };
            } else {
              comparator = new Comparator<Object>() {
                @Override
                public int compare(Object o1, Object o2) {
                  ConcurrentMap<String, Future<Object>> variableMap = Maps.newConcurrentMap();
                  variableMap.put(varA, Future.value(o1));
                  variableMap.put(varB, Future.value(o2));
                  Context newContext = Context.of(context, variableMap);
                  Future<Object> future = newContext.run(funcExtractor);
                  try {
                    Long result = (Long) Await.result(future);
                    return result.intValue();
                  } catch (Exception e) {
                    throw FunctionFailure.wrap(e, getAstNode(), context);
                  }
                }
              };
            }

            return context.run(collectionExtractor)
                .map(collectionObj -> {
                  try {
                    @SuppressWarnings("unchecked")
                    Collection<Object> collection = (Collection<Object>) collectionObj;
                    List<Object> list = new ArrayList<>(collection);
                    list.sort(comparator);
                    return list;
                  } catch (Exception e) {
                    throw FunctionFailure.wrap(e, getAstNode(), context);
                  }
                });
          }
        };
      }
    };
  }
}
