package com.twitter.botmaker.function;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.ConcurrentMap;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.DerivedFeature;
import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.botmaker.compiler.exceptions.FunctionFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.tuples.StackFrame;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Future;

public abstract class DerivedFeatureCall extends ASTNode<Runtime> {

  public static final DerivedFeature NAMESPACE;

  static {
    try {
      NAMESPACE = DerivedFeature.of(
          "DerivedFeature",
          "",
          "1",
          true
      );
    } catch (Exception e) {
      throw new RuntimeException("Failed to initialize DerivedFeature NAMESPACE", e);
    }
  }

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(ImmutableList.of(TP), TP);

  private final DerivedFeature derivedFeature;
  private final List<Pair<String, ASTNode>> variables;
  private final ASTNode bodyASTNode;
  private final boolean logged;
  private final int lineNo;
  private final Optional<Extractor<Runtime>> cachedExtractor;
  private final StackFrame stackFrame;

  public DerivedFeatureCall(
      DerivedFeature derivedFeature,
      boolean logged,
      int lineNo,
      List<Pair<String, ASTNode>> variables,
      ASTNode bodyASTNode) throws SemanticCheckFailure {
    super(derivedFeature.getName(), ImmutableList.of(bodyASTNode));
    this.derivedFeature = derivedFeature;
    this.logged = logged;
    this.variables = variables;
    this.bodyASTNode = bodyASTNode;
    this.lineNo = lineNo;
    this.cachedExtractor =
            variables.isEmpty() ? Optional.of(initializeExtractor()) : Optional.empty();

    this.stackFrame = new StackFrame(getName(), getLineNo());
  }

  public String getName() {
    return derivedFeature.getName();
  }

  public DerivedFeature getDerivedFeature() {
    return derivedFeature;
  }

  public List<Pair<String, ASTNode>> getVariables() {
    return variables;
  }

  public boolean isLogged() {
    return logged;
  }

  public int getLineNo() {
    return lineNo;
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  private Extractor<Runtime> initializeExtractor() {
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

    final String featureName = getName();
    final boolean toLogged = isLogged() && featureName != null && !featureName.isEmpty();

    if (variablesEx.size() > 0) {
      return new Extractor<Runtime>(this, childExtractors) {
        @Override
        public Future run(Context<Runtime> context) {
          ConcurrentMap<String, Future<Object>> newVariableTable = Maps.newConcurrentMap();
          for (Pair<String, Pair<Type, Extractor>> variablePair : variablesEx) {
            final Pair<Type, Extractor> extractorPair = variablePair.getSecond();
            final Extractor innerExtractor = extractorPair.getSecond();
            final Future value = context.run(innerExtractor);
            newVariableTable.put(variablePair.getFirst(), value);
          }
          Context newContext = Context.of(
              context,
              newVariableTable,
              stackFrame
          );
          Future<Object> result = newContext.run(bodyExtractor);
          if (!toLogged) {
            return result;
          }
          return result.map(feature -> {
            logFeature(context, featureName, bodyASTNode.getReturnType(), feature);
            return feature;
          });
        }
      };
    } else {
      return new Extractor<Runtime>(this, childExtractors) {
        @Override
        public Future run(Context<Runtime> context) {
          Context newContext = Context.of(context, stackFrame);
          Future<Object> result = newContext.run(bodyExtractor);
          if (!toLogged) {
            return result;
          }
          return result.map(feature -> {
            logFeature(context, featureName, bodyASTNode.getReturnType(), feature);
            return feature;
          });
        }
      };
    }
  }

  @Override
  public Extractor<Runtime> toExtractor() {
    return cachedExtractor.orElseGet(this::initializeExtractor);
  }

  private void logFeature(
      Context<Runtime> context, String featureName, Type featureType, Object featureValue) {
    try {
      context.addOutputFeature(
          derivedFeature,
          featureName,
          bodyASTNode.getReturnType(),
          featureValue);
    } catch (Exception ex) {
      throw new FunctionFailure(this, context.getStackFrames(), ex);
    }
  }

  public static DerivedFeatureCall of(
      DerivedFeature derivedFeature,
      boolean logged,
      int lineNo,
      List<Pair<String, ASTNode>> variables,
      ASTNode bodyASTNode
  ) throws SemanticCheckFailure {

    return new DerivedFeatureCall(derivedFeature, logged, lineNo, variables, bodyASTNode) {

      @Override
      protected CacheLevel getCacheLevel() {
        if (logged) {
          return CacheLevel.Event;
        } else {
          return CacheLevel.Global;
        }
      }

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
          } else if (childScope <= currentScope) {
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
