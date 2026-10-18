package com.twitter.botmaker.compiler;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

import com.google.common.base.Optional;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;

import org.apache.thrift.TBase;
import org.reflections.Reflections;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Tuple3;
import com.twitter.botmaker.compiler.types.StructTupleType;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.compiler.types.TypeContext;
import com.twitter.scrooge.ThriftStruct;

public class CompilerBuilder<E> {

  private static final class CompilerUnitBuilder {
    private final ImmutableMap.Builder<String, CompilerUnit> compilerUnitsBuilder =
        ImmutableMap.builder();
    private final String rootPackage;
    private final Class<?> runtimeClass;
    private Set<String> prefixes = new HashSet<>();

    private CompilerUnitBuilder(String rootPackage, Class<?> runtimeClass) {
      this.rootPackage = rootPackage;
      this.runtimeClass = runtimeClass;
    }

    public CompilerUnitBuilder put(String prefix) {
      prefixes.add(prefix);
      return this;
    }

    public CompilerUnitBuilder put(String k, CompilerUnit v) {
      compilerUnitsBuilder.put(k, v);
      return this;
    }

    public ImmutableMap<String, CompilerUnit> build() {
      Reflections reflections =
          ClassCache
              .REFLECTIONS_LOADING_CACHE
              .getUnchecked(new Tuple3(rootPackage, runtimeClass, prefixes));
      for (String prefix : prefixes) {
        for (Class<? extends ASTNode> clazz : ClassCache.forASTNodes(reflections)) {
          if (clazz.getName().startsWith(prefix)) {
            BotMakerFunction annotation = clazz.getAnnotation(BotMakerFunction.class);
            if (!annotation.deprecated()) {
              for (CompilerUnit c : CompilerUnit.of(clazz)) {
                compilerUnitsBuilder.put(c.funcName(), c);
              }
            }
          }
        }

        for (CompilerUnit cu : ClassCache.forCompilerUnits(reflections)) {
          if (cu.enabled() && cu.getClass().getName().startsWith(prefix)) {
            compilerUnitsBuilder.put(cu.funcName(), cu);
          }
        }
      }
      return compilerUnitsBuilder.build();
    }
  }

  private final ImmutableMap.Builder<String, ThriftType> thriftTypesBuilder;
  private final ImmutableMap.Builder<String, ThriftStructType> thriftStructsBuilder;
  private final ImmutableMap.Builder<String, StructTupleType> structTuplesBuilder;
  private final CompilerUnitBuilder compilerUnitsBuilder;
  private final ImmutableMap.Builder<String, Type> inputFeatureTypesBuilder;

  private boolean strictMode = false;

  public CompilerBuilder(Class<? extends E> runtimeClass) {
    this("com.twitter", runtimeClass);
  }

  public CompilerBuilder(String rootPackage, Class<? extends E> runtimeClass) {
    this.thriftTypesBuilder = ImmutableMap.builder();
    this.thriftStructsBuilder = ImmutableMap.builder();
    this.structTuplesBuilder = ImmutableMap.builder();
    this.compilerUnitsBuilder = new CompilerUnitBuilder(rootPackage, runtimeClass);
    this.inputFeatureTypesBuilder = ImmutableMap.builder();
  }

  public CompilerBuilder<E> addThriftType(Class<? extends TBase> clazz) {
    return addThriftType(clazz, clazz.getSimpleName());
  }

  public CompilerBuilder<E> addThriftType(Class<? extends TBase> clazz, String typeName) {

    ThriftType thriftType = Type.thriftOf(clazz);
    thriftTypesBuilder.put(typeName, thriftType);
    for (CompilerUnit c : CompilerUnit.of(typeName, thriftType)) {
      compilerUnitsBuilder.put(c.funcName(), c);
    }

    return this;
  }

  public CompilerBuilder<E> addThriftStructType(
      Class<? extends ThriftStruct> clazz, String typeName) {
    ThriftStructType thriftType = Type.thriftStructOf(clazz);
    thriftStructsBuilder.put(typeName, thriftType);
    for (CompilerUnit c : CompilerUnit.of(typeName, thriftType)) {
      compilerUnitsBuilder.put(c.funcName(), c);
    }

    return this;
  }

  public CompilerBuilder<E> addStructTupleType(StructTupleType structTupleType, String typeName) {

    structTuplesBuilder.put(typeName, structTupleType);
    for (CompilerUnit c : CompilerUnit.of(typeName, structTupleType)) {
      compilerUnitsBuilder.put(c.funcName(), c);
    }

    return this;
  }

  public CompilerBuilder<E> addInputFeature(String name, Type type) {
    inputFeatureTypesBuilder.put(name, type);
    return this;
  }

  public CompilerBuilder<E> addInputFeatures(Map<String, Type> features) {
    for (Map.Entry<String, Type> entry : features.entrySet()) {
      inputFeatureTypesBuilder.put(entry.getKey(), entry.getValue());
    }

    return this;
  }

  public CompilerBuilder<E> strictMode() {
    strictMode = true;
    return this;
  }

  public CompilerBuilder<E> addPackage(String packageName) {
    String prefix = packageName.endsWith(".") ? packageName : packageName + ".";
    this.compilerUnitsBuilder.put(prefix);
    return this;
  }

  public CompilerBuilder<E> addCompilerUnit(CompilerUnit compilerUnit) {
    this.compilerUnitsBuilder.put(compilerUnit.funcName(), compilerUnit);
    return this;
  }

  public TypeContext typeContext() {

    ImmutableMap<String, ThriftType> thriftTypes = thriftTypesBuilder.build();
    ImmutableMap<String, ThriftStructType> thriftStructTypes = thriftStructsBuilder.build();
    ImmutableMap<String, StructTupleType> structTupleTypes = structTuplesBuilder.build();

    return new TypeContext() {

      @Override
      public Optional<Type> getTypeFromName(String typeName) {
        if (thriftTypes.containsKey(typeName)) {
          return Optional.of(thriftTypes.get(typeName));
        } else if (thriftStructTypes.containsKey(typeName)) {
          return Optional.of(thriftStructTypes.get(typeName));
        } else if (structTupleTypes.containsKey(typeName)) {
          return Optional.of(structTupleTypes.get(typeName));
        } else {
          return Optional.absent();
        }
      }
    };
  }

  public Compiler<E> build() {

    ImmutableMap<String, CompilerUnit> compilerUnits = compilerUnitsBuilder.build();
    ImmutableMap<String, Type> inputFeatureTypes = inputFeatureTypesBuilder.build();

    ImmutableSet.Builder<Fingerprint> fingerprintBuilder = ImmutableSet.builder();
    for (CompilerUnit compilerUnit : compilerUnits.values()) {
      fingerprintBuilder.add(compilerUnit.fingerprint());
    }
    ImmutableSet<Fingerprint> compilerUnitsFingerprints = fingerprintBuilder.build();

    TypeContext typeContext = typeContext();

    return new Compiler<E>(typeContext) {

      @Override
      public List<BotMakerDoc> getBotMakerDocs() {
        ImmutableList.Builder<BotMakerDoc> builder = ImmutableList.builder();
        builder.addAll(super.getBotMakerDocs());
        for (CompilerUnit compilerUnit : compilerUnits.values()) {
          BotMakerDoc doc = compilerUnit.botMakerDoc();
          if (doc != BotMakerDoc.EMPTY) {
            builder.add(doc);
          }
        }

        return builder.build();
      }

      @Override
      public boolean containsFingerprint(Fingerprint fingerprint) {
        return super.containsFingerprint(fingerprint)
            || compilerUnitsFingerprints.contains(fingerprint);
      }

      @Override
      protected Type getInputFeatureType(
          String namespace,
          String featureName
      ) throws SemanticCheckFailure {
        String fullFeatureName = namespace.isEmpty()
            ? featureName : String.format("%s.%s", namespace, featureName);

        if (inputFeatureTypes.containsKey(fullFeatureName)) {
          return inputFeatureTypes.get(fullFeatureName);
        } else if (!strictMode) {
          return Type.OBJECT;
        } else {
          throw new SemanticCheckFailure(
              String.format("Input Feature %s does not exist", fullFeatureName));
        }
      }

      @Override
      protected boolean containsFunction(String name) {
        return compilerUnits.containsKey(name);
      }

      @Override
      protected ASTNode createFunctionASTNode(
          CompilerContext<E> context,
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
    };
  }
}
