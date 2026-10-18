package com.twitter.botmaker.compiler;

import java.util.Map;
import java.util.function.BiPredicate;

import com.google.common.base.Optional;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Maps;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.compiler.types.TypeContext;

class CompilerScope implements TypeContext {
  private final long id;
  private final TypeContext surrounding;

  private final Map<String, Parameter> variables = Maps.newConcurrentMap();
  private final Map<String, ThriftType> thriftTypes = Maps.newConcurrentMap();
  private final Map<String, ThriftStructType> thriftStructTypes = Maps.newConcurrentMap();
  private final Map<String, CompilerUnit> compilerUnits = Maps.newConcurrentMap();

  public CompilerScope(long id, TypeContext surrounding) {
    this.id = id;
    this.surrounding = surrounding;
  }

  protected long getId() {
    return id;
  }

  Map<String, Parameter> getVariables() {
    return variables;
  }

  private static <T>  void checkAndMergeMap(
      Map<String, T> thisMap,
      Map<String, T> thatMap,
      String elementName,
      BiPredicate<T, T> predicate
  ) throws SemanticCheckFailure {
    for (Map.Entry<String, T> entry : thatMap.entrySet()) {
      if (!thisMap.containsKey(entry.getKey())) {
        thisMap.put(entry.getKey(), entry.getValue());
        continue;
      }
      if (predicate.test(thisMap.get(entry.getKey()), entry.getValue())) {
        throw new SemanticCheckFailure(String.format(
            "%s %s already exists in the same scope and is defined differently",
            elementName, entry.getKey()));
      }
    }
  }

  public void mergeTypes(CompilerScope that) throws SemanticCheckFailure {
    checkAndMergeMap(
            this.thriftTypes,
            that.thriftTypes,
            "thrift type",
            ThriftType.EQUALITY_CHECK
    );
    checkAndMergeMap(
            this.thriftStructTypes,
            that.thriftStructTypes,
            "thrift struct type",
            ThriftStructType.EQUALITY_CHECK
    );
    checkAndMergeMap(
            this.compilerUnits,
            that.compilerUnits,
            "compiler unit",
            CompilerUnit.EQUALITY_CHECK
    );
  }

  void defineVariable(Parameter parameter) throws SemanticCheckFailure {
    if (variables.containsKey(parameter.getParameterName())) {
      throw new SemanticCheckFailure(
          String.format("variable %s already exists in the same scope",
              parameter.getParameterName()));
    }
    variables.put(parameter.getParameterName(), parameter);
  }

  boolean isVariableDefined(String variableText) {
    return variables.containsKey(variableText);
  }

  Parameter getVariable(String variableText) {
    return variables.get(variableText);
  }

  void defineType(ThriftType thriftType, String alias) {
    thriftTypes.put(alias, thriftType);
    for (CompilerUnit c : CompilerUnit.of(alias, thriftType)) {
      compilerUnits.put(c.funcName(), c);
    }
  }

  void defineType(ThriftStructType thriftStructType, String alias) {
    thriftStructTypes.put(alias, thriftStructType);
    for (CompilerUnit c : CompilerUnit.of(alias, thriftStructType)) {
      compilerUnits.put(c.funcName(), c);
    }
  }

  boolean containsFunction(String name) {
    return compilerUnits.containsKey(name);
  }

  ASTNode createFunctionASTNode(
      CompilerContext context,
      String funcName,
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    if (!compilerUnits.containsKey(funcName)) {
      throw new SemanticCheckFailure(String.format("Function '%s' does not exist", funcName));
    }

    CompilerUnit compilerUnit = compilerUnits.get(funcName);
    if (compilerUnit.fingerprint() != Fingerprint.EMPTY) {
      context.trackUsage(compilerUnit);
    }

    return compilerUnit.createASTNode(children);
  }

  @Override
  public Optional<Type> getTypeFromName(String typeName) {
    if (thriftTypes.containsKey(typeName)) {
      return Optional.of(thriftTypes.get(typeName));
    } else if (thriftStructTypes.containsKey(typeName)) {
      return Optional.of(thriftStructTypes.get(typeName));
    } else {
      return surrounding.getTypeFromName(typeName);
    }
  }
}
