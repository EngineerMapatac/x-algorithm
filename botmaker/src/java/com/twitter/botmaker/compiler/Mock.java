package com.twitter.botmaker.compiler;

import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.ASTNode.Signature;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public interface Mock {

  ASTNode<Runtime> createMock(ASTNode target) throws SemanticCheckFailure;

  static Mock of(String exprText, String varName, ASTNode bodyNode) {

    return new Mock() {

      public ASTNode<Runtime> createMock(ASTNode target) throws SemanticCheckFailure {

        if (Type.isDivergentTo(target.getReturnType(), bodyNode.getReturnType())) {
          throw new SemanticCheckFailure(
              String.format("Mock target type %s is not compatible the mock type %s.",
                  target.getReturnType(),
                  bodyNode.getReturnType()
              ));
        }

        Signature signature = new ASTNode.Signature(
            Type.OBJECT,
            target.getReturnType()
        );

        ImmutableList.Builder<ASTNode> childAstNodes = ImmutableList.builder();
        childAstNodes.addAll(target.getChildren());
        childAstNodes.add(bodyNode);

        return new ASTNode<Runtime>(exprText, childAstNodes.build()) {

          @Override
          protected CacheLevel getCacheLevel() {
            return CacheLevel.Never;
          }

          @Override
          protected String computeClassName() {
            return "Mock";
          }

          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          public Extractor toExtractor() {

            Extractor targetExtractor = target.toExtractor();
            ImmutableList<Extractor> children = targetExtractor.getChildren();

            Extractor bodyExtractor = bodyNode.toExtractor();

            return new Extractor(this, children) {
              @Override
              public Future<Object> run(Context context) {

                ImmutableList.Builder<Future<Object>> argBuilder = ImmutableList.builder();
                for (int i = 0; i < children.size(); i++) {
                  Future arg = context.run(children.get(i));
                  argBuilder.add(arg);
                }

                Future<Object> args = Future
                    .collect(argBuilder.build())
                    .map(Function.func(list -> (Object) list));

                ConcurrentMap<String, Future<Object>> newVariableTable = Maps.newConcurrentMap();
                newVariableTable.put(varName, args);
                Context newContext = Context.of(context, newVariableTable);
                return newContext.run(bodyExtractor);
              }
            };
          }
        };
      }
    };
  }

  static Mock of(String exprText, ASTNode bodyNode) {
    return new Mock() {

      public ASTNode<Runtime> createMock(ASTNode target) throws SemanticCheckFailure {

        if (Type.isDivergentTo(target.getReturnType(), bodyNode.getReturnType())) {
          throw new SemanticCheckFailure(
              String.format("Mock target type %s is not compatible the mock type %s.",
                  target.getReturnType(),
                  bodyNode.getReturnType()
              ));
        }

        Signature signature = new ASTNode.Signature(
            Type.OBJECT,
            target.getReturnType()
        );

        ImmutableList.Builder<ASTNode> childAstNodes = ImmutableList.builder();
        childAstNodes.addAll(target.getChildren());
        childAstNodes.add(bodyNode);

        return new ASTNode<Runtime>(exprText, childAstNodes.build()) {

          @Override
          protected CacheLevel getCacheLevel() {
            return CacheLevel.Never;
          }

          @Override
          protected String computeClassName() {
            return "Mock";
          }

          @Override
          public Signature getSignature() {
            return signature;
          }

          @Override
          public Extractor toExtractor() {

            Extractor targetExtractor = target.toExtractor();
            ImmutableList<Extractor> children = targetExtractor.getChildren();

            Extractor bodyExtractor = bodyNode.toExtractor();

            return new Extractor(this, children) {
              @Override
              public Future run(Context context) {

                ImmutableList.Builder<Future<Object>> argBuilder = ImmutableList.builder();
                for (int i = 0; i < children.size(); i++) {
                  Future arg = context.run(children.get(i));
                  argBuilder.add(arg);
                }

                return Future
                    .collect(argBuilder.build())
                    .flatMap(Function.func(list -> context.run(bodyExtractor)));
              }
            };
          }
        };
      }
    };
  }
}
