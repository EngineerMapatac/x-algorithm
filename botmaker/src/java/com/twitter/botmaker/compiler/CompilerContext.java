package com.twitter.botmaker.compiler;

import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicLong;

import javax.annotation.concurrent.NotThreadSafe;

import com.google.common.base.Optional;
import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.google.common.collect.Maps;
import com.google.common.collect.Sets;

import org.antlr.runtime.tree.Tree;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.exceptions.ParseFailure;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.tuples.Tuple3;
import com.twitter.botmaker.compiler.types.ThriftStructType;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.compiler.types.TypeCompiler;
import com.twitter.botmaker.compiler.types.TypeContext;
import com.twitter.botmaker.function.Variable;
import com.twitter.botmaker.function.feature.InputFeature;
import com.twitter.botmaker.function.thrift.ThriftOperator;
import com.twitter.botmaker.function.thrift.ThriftStructOperator;

@NotThreadSafe
public class CompilerContext<E> {

  private static final AtomicLong NEXT_CONTEXT_ID = new AtomicLong(0);
  private static final String GLOBAL_INPUT_FEATURE_NAMESPACE = "";

  private final Compiler<E> compiler;
  private final TypeContext typeContext;
  private final DerivedFeatureProvider derivedFeatures;
  private final String inputFeatureNameSpace;
  private final boolean enableContextScope;

  private final Map<String, Pair<ASTNode<E>, CompilerContext<E>>> derivedFeatureCache;

  private final LinkedList<CompilerScope> scopeList = Lists.newLinkedList();
  private final LinkedList<TestScope> testScopeList = Lists.newLinkedList();

  private final Set<String> curExpandedMacros = Sets.newHashSet();
  private final Set<UsageUnit> usageTracker = Sets.newHashSet();
  private final Set<CompilerContext<E>> mergedVirtualContexts = Sets.newHashSet();

  CompilerContext(
      Compiler<E> compiler,
      TypeContext typeContext,
      DerivedFeatureProvider derivedFeatures,
      String inputFeatureNameSpace,
      boolean enableContextScope,
      Map<String, Pair<ASTNode<E>, CompilerContext<E>>> derivedFeatureCache
  ) {
    this.compiler = compiler;
    this.typeContext = typeContext;
    this.derivedFeatures = derivedFeatures;
    this.inputFeatureNameSpace = inputFeatureNameSpace;
    this.enableContextScope = enableContextScope;
    this.derivedFeatureCache = derivedFeatureCache;

    scopeList.addFirst(new CompilerScope(NEXT_CONTEXT_ID.getAndIncrement(), typeContext));

    testScopeList.addFirst(new TestScope() {
      @Override
      public ASTNode createMock(ASTNode target) {
        return target;
      }
    });
  }

  @SuppressWarnings("unchecked")
  public ASTNode<E> getCachedDFBody(
      DerivedFeature derivedFeature
  ) throws ParseFailure, SemanticCheckFailure {
    if (enableContextScope || !derivedFeature.getParams().isEmpty()) {
      return compiler.createASTNodeTree(this, derivedFeature.getTree());
    }
    if (derivedFeatureCache.containsKey(derivedFeature.getName())) {
      Pair<ASTNode<E>, CompilerContext<E>> pair =
          derivedFeatureCache.get(derivedFeature.getName());
      mergeVirtualContext(pair.getSecond());
      return pair.getFirst();
    }
    CompilerContext<E> virtualContext = new Builder<E>()
        .setCompiler(this.compiler)
        .setTypeContext(this.typeContext)
        .setDerivedFeatures(this.derivedFeatures)
        .setInputFeatureNamespace(this.inputFeatureNameSpace)
        .enableContextScope(false)
        .setDerivedFeatureCache(this.derivedFeatureCache)
        .build();
    ASTNode<E> bodyNode = compiler.createASTNodeTree(virtualContext, derivedFeature.getTree());
    derivedFeatureCache.put(derivedFeature.getName(), Pair.of(bodyNode, virtualContext));
    mergeVirtualContext(virtualContext);
    return bodyNode;
  }

  private void mergeVirtualContext(CompilerContext<E> virtualContext) throws SemanticCheckFailure {
    if (mergedVirtualContexts.contains(virtualContext)) {
      return;
    }
    this.currentScope().mergeTypes(virtualContext.currentScope());
    this.usageTracker.addAll(virtualContext.usageTracker);
    this.mergedVirtualContexts.add(virtualContext);
  }

  public ASTNode<E> compile(String expression) throws ParseFailure, SemanticCheckFailure {
    Tree tree = Parser.createParseTree(expression);
    return compile(tree, ImmutableList.of(), Optional.absent());
  }

  public ASTNode<E> compileWithParams(String expression) throws ParseFailure, SemanticCheckFailure {
    Tuple3<Tree, ImmutableList<Tuple3<Tree, String, Optional<Tree>>>, Optional<Tree>> result =
        Parser.createParseParamsTree(expression);
    return compile(result.getFirst(), result.getSecond(), result.getThird());
  }

  private ASTNode<E> compile(
      Tree tree,
      ImmutableList<Tuple3<Tree, String, Optional<Tree>>> params,
      Optional<Tree> returnType)
      throws ParseFailure, SemanticCheckFailure {
    if (params.size() > 0) {
      CompilerScope rootScope = addScope();
      for (Tuple3<Tree, String, Optional<Tree>> param : params) {
        String paramName = param.getSecond();
        Type paramType = TypeCompiler.getType(param.getFirst(), currentScope());
        rootScope.defineVariable(Parameter.of(paramName, paramType));
        if (param.getThird().isPresent()) {
          ASTNode<E> node = compiler.createASTNodeTree(this, param.getThird().get());
          if (Type.isDivergentTo(paramType, node.getReturnType())) {
            throw new SemanticCheckFailure(String.format(
                "Parameter [%s] type [%s] is not compatible with the default value type [%s]",
                paramName,
                paramType.toString(),
                node.getReturnType().toString()
            ));
          }
        }
      }
    }

    ASTNode<E> astNode = compiler.createASTNodeTree(this, tree);
    if (returnType.isPresent()) {
      Type claimedType = TypeCompiler.getType(returnType.get(), currentScope());
      if (Type.isDivergentTo(claimedType, astNode.getReturnType())) {
        throw new SemanticCheckFailure(String.format(
            "ASTNode return type [%s] is not compatible with the claimed return type [%s]",
            astNode.getReturnType().toString(),
            claimedType.toString()));
      }
    }

    return astNode;
  }

  public boolean containsFingerprint(Fingerprint fingerprint) {
    return compiler.containsFingerprint(fingerprint)
        || derivedFeatures.containsFingerprint(fingerprint);
  }

  public List<BotMakerDoc> getDescriptions() {
    ImmutableList.Builder<BotMakerDoc> builder = ImmutableList.builder();
    builder.addAll(compiler.getBotMakerDocs());
    for (DerivedFeature derivedFeature : derivedFeatures.all()) {
      builder.add(BotMakerDoc.of(derivedFeature));
    }
    return builder.build();
  }

  public String getInputFeatureNameSpace() {
    return inputFeatureNameSpace;
  }

  public Set<UsageUnit> getUsageTracker() {
    return usageTracker;
  }

  Optional<DerivedFeature> getDerivedFeature(String derivedFeatureName) {
    return derivedFeatures.get(derivedFeatureName);
  }

  void trackUsage(DerivedFeature derivedFeature) {
    usageTracker.add(UsageUnit.ofDerivedFeatureFingerprint(derivedFeature.getFingerprint()));
  }

  void trackUsage(InputFeature inputFeature) {
    usageTracker.add(UsageUnit.ofInputFeatureName(inputFeature.getExprText()));
  }

  void trackUsage(CompilerUnit compilerUnit) {
    usageTracker.add(UsageUnit.ofCompilerUnitFingerprint(compilerUnit.fingerprint()));
  }

  void addMacro(String funcName) {
    curExpandedMacros.add(funcName);
  }

  void removeMacro(String funcName) {
    curExpandedMacros.remove(funcName);
  }

  boolean containsMacro(String funcName) {
    return curExpandedMacros.contains(funcName);
  }

  public void clearMacros() {
    curExpandedMacros.clear();
  }

  long currentScopeId() throws SemanticCheckFailure {
    if (scopeList.isEmpty()) {
      throw new SemanticCheckFailure("There is no scope");
    }
    if (enableContextScope) {
      return currentScope().getId();
    } else {
      return ASTNode.CacheLevel.ConstantContextScope.scope;
    }
  }

  private CompilerScope currentScope() throws SemanticCheckFailure {
    if (scopeList.isEmpty()) {
      throw new SemanticCheckFailure("There is no scope");
    }
    return scopeList.getFirst();
  }

  CompilerScope addScope() throws SemanticCheckFailure {
    CompilerScope scope =
        new CompilerScope(NEXT_CONTEXT_ID.getAndIncrement(), currentScope());
    scopeList.addFirst(scope);
    return scope;
  }

  void removeScope() {
    scopeList.removeFirst();
  }

  void defineVariable(
      String variableName,
      Type returnType,
      ASTNode astNode) throws SemanticCheckFailure {

    if (scopeList.size() <= 1) {
      throw new SemanticCheckFailure(
          String.format("variable %s cannot be defined: there is no scope", variableName));
    }

    CompilerScope currentScope = scopeList.getFirst();
    Parameter parameter = Parameter.variable(variableName, returnType, astNode);
    currentScope.defineVariable(parameter);
  }

  void defineVariable(
      String variableName,
      ASTNode astNode) throws SemanticCheckFailure {
    defineVariable(variableName, astNode.getReturnType(), astNode);
  }

  Variable getVariable(String variableText) throws SemanticCheckFailure {
    for (CompilerScope scope : scopeList) {
      if (scope.isVariableDefined(variableText)) {
        Parameter parameter = scope.getVariable(variableText);
        return new Variable(variableText, parameter);
      }
    }
    throw new SemanticCheckFailure(String.format("variable %s not defined", variableText));
  }

  TypeContext getTypeContext() throws SemanticCheckFailure {
    if (scopeList.isEmpty()) {
      throw new SemanticCheckFailure("There is no type context");
    }

    return scopeList.getFirst();
  }

  void defineType(ThriftType thriftType, String alias) throws SemanticCheckFailure {
    if (scopeList.size() <= 1) {
      throw new SemanticCheckFailure(
          String.format("type cannot defined in root scope: %s", alias));
    }

    CompilerScope scope = currentScope();
    scope.defineType(thriftType, alias);
  }

  void defineType(ThriftStructType scroogeType, String alias) throws SemanticCheckFailure {
    if (scopeList.size() <= 1) {
      throw new SemanticCheckFailure(
          String.format("type cannot defined in root scope: %s", alias));
    }

    CompilerScope scope = currentScope();
    scope.defineType(scroogeType, alias);
  }

  TestScope addTestScope() {
    TestScope testScope = new TestScope();
    testScopeList.addFirst(testScope);
    return testScope;
  }

  void removeTestScope() {
    testScopeList.removeFirst();
  }

  void addMockPass(String funcName) {
    testScopeList.getFirst().addMockPass(funcName);
  }

  void addMock(String funcName, ASTNode bodyNode) {
    testScopeList.getFirst().addMock(funcName, bodyNode);
  }

  void addMock(String funcName, String varName, ASTNode bodyNode) {
    testScopeList.getFirst().addMock(funcName, varName, bodyNode);
  }

  void addMockPackage(String packageName) {
    testScopeList.getFirst().addMockPackage(packageName);
  }

  boolean containsFunction(String funcName) {
    for (CompilerScope scope : scopeList) {
      if (scope.containsFunction(funcName)) {
        return true;
      }
    }

    return compiler.containsFunction(funcName);
  }

  ASTNode<E> createFunctionASTNode(
      String funcName,
      ImmutableList<ASTNode> children) throws SemanticCheckFailure {

    for (CompilerScope scope : scopeList) {
      if (scope.containsFunction(funcName)) {
        return scope.createFunctionASTNode(this, funcName, children);
      }
    }

    ASTNode astNode = compiler.createFunctionASTNode(this, funcName, children);
    TestScope testScope = testScopeList.getFirst();
    return testScope.createMock(astNode);
  }

  public static class Builder<T> {
    private Compiler<T> compiler;
    private TypeContext typeContext;
    private DerivedFeatureProvider derivedFeatures;
    private String inputFeatureNamespace = GLOBAL_INPUT_FEATURE_NAMESPACE;
    private boolean enableContextScope = true;
    private Map<String, Pair<ASTNode<T>, CompilerContext<T>>> derivedFeatureCache = new HashMap<>();

    public Builder<T> setCompiler(Compiler<T> thatCompiler) {
      this.compiler = thatCompiler;
      return this;
    }

    public Builder<T> setTypeContext(TypeContext thatTypeContext) {
      this.typeContext = thatTypeContext;
      return this;
    }

    public Builder<T> setDerivedFeatures(DerivedFeatureProvider thatDerivedFeatures) {
      this.derivedFeatures = thatDerivedFeatures;
      return this;
    }

    public Builder<T> setInputFeatureNamespace(String thatInputFeatureNamespace) {
      this.inputFeatureNamespace = thatInputFeatureNamespace;
      return this;
    }

    public Builder<T> enableContextScope(boolean thatEnableAstCacheLevel) {
      this.enableContextScope = thatEnableAstCacheLevel;
      return this;
    }

    public Builder<T> setDerivedFeatureCache(
        Map<String, Pair<ASTNode<T>, CompilerContext<T>>> thatDerivedFeatureCache) {
      this.derivedFeatureCache = thatDerivedFeatureCache;
      return this;
    }

    public CompilerContext<T> build() {
      Preconditions.checkNotNull(compiler);
      Preconditions.checkNotNull(typeContext);
      Preconditions.checkNotNull(derivedFeatures);
      return new CompilerContext<T>(
          compiler,
          typeContext,
          derivedFeatures,
          inputFeatureNamespace,
          enableContextScope,
          derivedFeatureCache
      );
    }
  }

  static class TestScope {

    private final Map<String, Mock> mocks = Maps.newConcurrentMap();
    private final Set<String> funcs = Sets.newConcurrentHashSet();
    private final Set<String> packages = Sets.newConcurrentHashSet();

    public TestScope() {
      packages.add("com.twitter.botmaker.function");
      packages.add("com.twitter.botmaker.compiler");
    }

    public void addMock(String funcName, String varName, ASTNode bodyNode) {
      Mock mock = Mock.of(funcName, varName, bodyNode);
      mocks.put(funcName, mock);
    }

    public void addMock(String funcName, ASTNode bodyNode) {
      Mock mock = Mock.of(funcName, bodyNode);
      mocks.put(funcName, mock);
    }

    public void addMockPass(String funcName) {
      funcs.add(funcName);
    }

    public void addMockPackage(String packageName) {
      packages.add(packageName);
    }

    public ASTNode createMock(ASTNode target) throws SemanticCheckFailure {

      String className = target.getClassName();

      if (mocks.containsKey(className)) {
        return mocks.get(className).createMock(target);
      }

      if (target instanceof ThriftOperator) {
        return target;
      }

      if (target instanceof ThriftStructOperator) {
        return target;
      }

      String nls = target.getPackageName();
      for (String filter : packages) {
        if (nls.startsWith(filter)) {
          return target;
        }
      }

      if (funcs.contains(className)) {
        return target;
      }

      throw new SemanticCheckFailure(
          String.format("The mock for function %s is not defined.", target.getClassName())
      );
    }
  }
}


