package com.twitter.botmaker.function;

import java.util.List;
import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Future;

public abstract class With extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(ImmutableList.of(TP), TP);

  private final ImmutableList<Pair<String, ASTNode>> variables;
  private final ASTNode bodyASTNode;

  public With(
      String exprText,
      List<Pair<String, ASTNode>> variables,
      ASTNode bodyASTNode) throws SemanticCheckFailure {
    super(exprText, ImmutableList.of(bodyASTNode));

    this.variables = internVariableNames(variables);
    this.bodyASTNode = bodyASTNode;
  }

  private static ImmutableList<Pair<String, ASTNode>> internVariableNames(
      List<Pair<String, ASTNode>> variableList) {
    List<Pair<String, ASTNode>> accumulation = Lists.newArrayListWithCapacity(variableList.size());
    for (Pair<String, ASTNode> oldPair : variableList) {
      accumulation.add(new Pair<>(intern(oldPair.getFirst()), oldPair.getSecond()));
    }
    return ImmutableList.copyOf(accumulation);
  }

  public List<Pair<String, ASTNode>> getVariables() {
    return variables;
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

    final ImmutableList.Builder<Extractor> extractorsBuilder = new ImmutableList.Builder<>();
    final ImmutableList.Builder<Pair<String, Pair<Type, Extractor>>> variablesBuilder =
        new ImmutableList.Builder<>();
    for (Pair<String, ASTNode> variablePair : variables) {
      ASTNode node = variablePair.getSecond();
      Extractor extractor = node.toExtractor();
      Pair<Type, Extractor> extractorPair = Pair.of(
          node.getReturnType(),
          extractor);
      Pair<String, Pair<Type, Extractor>> extractorWithName =
          Pair.of(variablePair.getFirst(), extractorPair);

      extractorsBuilder.add(extractor);
      variablesBuilder.add(extractorWithName);
    }

    final Extractor bodyExtractor = bodyASTNode.toExtractor();
    extractorsBuilder.add(bodyExtractor);

    final ImmutableList<Extractor> childExtractors = extractorsBuilder.build();
    final ImmutableList<Pair<String, Pair<Type, Extractor>>> variablesEx =
        variablesBuilder.build();

    return new Extractor<Runtime>(this, childExtractors) {

      @Override
      public Future run(Context<Runtime> context) {

        ConcurrentMap<String, Future<Object>> newVariableTable = Maps.newConcurrentMap();
        Context currentContext = context;
        for (Pair<String, Pair<Type, Extractor>> variablePair : variablesEx) {
          final Pair<Type, Extractor> extractorPair = variablePair.getSecond();
          final Extractor innerExtractor = extractorPair.getSecond();

          final Future<Object> value = currentContext.run(innerExtractor);

          newVariableTable.put(variablePair.getFirst(), value);
          currentContext = Context.of(currentContext, newVariableTable);
        }

        return currentContext.run(bodyExtractor);
      }
    };
  }

  public static With of(
      String exprText,
      List<Pair<String, ASTNode>> variables,
      ASTNode bodyASTNode) throws SemanticCheckFailure {

    return new With(exprText, variables, bodyASTNode) {
      @Override
      protected void computeFingerprint(Hasher hasher) {
        super.computeFingerprint(hasher);

        hasher.putInt(variables.size());
        for (Pair<String, ASTNode> variable : variables) {
          Fingerprint fp = variable.getSecond().getFingerprint();
          fp.computerFingerprint(hasher);
        }
      }

      @Override
      public long computeFingerprintScope(long currentScope) {
        long scope = super.computeFingerprintScope(currentScope);
        if (scope == CacheLevel.Never.scope) {
          return scope;
        }

        for (Pair<String, ASTNode> variable : variables) {
          ASTNode child = variable.getSecond();
          long childScope = child.getFingerprint().scope;
          if (childScope == CacheLevel.Never.scope) {
            return childScope;
          } else if (childScope < currentScope) {
            scope = Math.max(scope, childScope);
          } else {
            scope = Math.max(scope, child.computeFingerprintScope(currentScope));
          }
        }
        return scope;
      }
    };
  }
}
