package com.twitter.botmaker;

import java.util.List;
import java.util.Map;
import java.util.Objects;

import scala.PartialFunction;
import scala.Unit;
import scala.runtime.BoxedUnit;

import com.google.common.base.Optional;
import com.google.common.base.Preconditions;
import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Interner;
import com.google.common.collect.Interners;
import com.google.common.collect.Maps;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.compiler.Fingerprint;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.Constant;

public abstract class ASTNode<E> {

  public ASTNode(
      String exprText,
      List<ASTNode> astNodeChildren
  ) throws SemanticCheckFailure {

    this.exprText = intern(exprText);
    this.astNodeChildren = astNodeChildren != null
        ? ImmutableList.copyOf(astNodeChildren) : ImmutableList.of();
    this.signature = getSignature();

    validateSignature();
    this.returnType = inferReturnTypeAndDoTypeChecking();
    this.className = computeClassName();
    this.packageName = computePackageName();
    this.fingerprint = new Fingerprint(
        computeFingerprintScope(CacheLevel.Max.scope),
        computeFingerprint());
  }

  protected ASTNode(
      String exprText,
      ImmutableList<ASTNode> astNodeChildren,
      Signature signature,
      Type returnType,
      byte[] fingerprintBytes) {
    this.exprText = intern(exprText);
    this.astNodeChildren = astNodeChildren != null ? astNodeChildren : ImmutableList.of();
    this.signature = signature;
    this.returnType = returnType;
    this.className = computeClassName();
    this.packageName = computePackageName();
    this.fingerprint = new Fingerprint(
        computeFingerprintScope(CacheLevel.Max.scope),
        fingerprintBytes);
  }

  private static final Interner<String> STRING_INTERNER = Interners.newWeakInterner();
  private final String exprText;
  private final ImmutableList<ASTNode> astNodeChildren;
  private final Signature signature;
  private final Type returnType;
  private final String className;
  private final String packageName;
  private final Fingerprint fingerprint;
  protected static String intern(String str) {
    return str != null ? STRING_INTERNER.intern(str) : null;
  }

  public final String getExprText() {
    return exprText;
  }

  public final String getClassName() {
    return className;
  }

  public final String getPackageName() {
    return packageName;
  }

  public final List<ASTNode> getChildren() {
    return astNodeChildren;
  }

  public final Type getReturnType() {
    return returnType;
  }

  public final Fingerprint getFingerprint() {
    return fingerprint;
  }

  protected abstract CacheLevel getCacheLevel();

  protected final byte[] computeFingerprint() {
    Hasher hasher = Fingerprint.newHasher();
    computeFingerprint(hasher);
    return hasher.hash().asBytes();
  }

  protected void computeFingerprint(Hasher hasher) {
    signature.computerFingerprint(hasher);
    hasher.putUnencodedChars(Strings.nullToEmpty(exprText));
    hasher.putUnencodedChars(this.className);
    hasher.putUnencodedChars(this.packageName);

    hasher.putInt(astNodeChildren.size());
    for (ASTNode child : astNodeChildren) {
      Fingerprint fp = child.getFingerprint();
      fp.computerFingerprint(hasher);
    }
  }

  protected String computeClassName() {
    String name = this.getClass().getSimpleName();
    if (Strings.isNullOrEmpty(name)) {
      name = this.getClass().getSuperclass().getSimpleName();
      if (Strings.isNullOrEmpty(name)) {
        name = exprText;
      }
    }
    return intern(name);
  }

  protected String computePackageName() {
    return this.getClass().getPackage().getName();
  }

  public long computeFingerprintScope(long currentScope) {

    long scope = getCacheLevel().scope;
    if (scope == CacheLevel.Never.scope) {
      return scope;
    }

    for (ASTNode child : astNodeChildren) {
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

  public static void assertChildConstant(
      String exprText, ImmutableList<ASTNode> children, int index) throws SemanticCheckFailure {
    ASTNode node = children.get(index);
    if (!Constant.class.isAssignableFrom(node.getClass())) {
      throw new SemanticCheckFailure(String.format(
          "type checking expression %s failed: invalid argument type: expected a constant %s",
          exprText,
          node.getReturnType().toString())
      );
    }
  }

  public static void assertChildrenSize(
      String exprText, ImmutableList<ASTNode> children, int expected) throws SemanticCheckFailure {
    if (children.size() != expected) {
      throw new SemanticCheckFailure(String.format(
          "ASTNode %s expected %d arguments, %d passed.", exprText, expected, children.size()));
    }

  }

  public static void assertChildrenSize(
      String exprText, ImmutableList<ASTNode> children,
      int min, int max) throws SemanticCheckFailure {
    if (children.size() < min || children.size() > max) {
      throw new SemanticCheckFailure(String.format(
          "ASTNode %s expected %d to %d arguments, %d passed.",
          exprText, min, max, children.size()));
    }
  }

  public abstract Signature getSignature();

  public abstract Extractor<E> toExtractor();

  protected final BoxedUnit unit() {
    return BoxedUnit.UNIT;
  }

  protected final ImmutableList<Extractor> buildExtractorsOfChildren() {
    ImmutableList.Builder<Extractor> builder = ImmutableList.builder();
    for (ASTNode<E> node : getChildren()) {
      builder.add(node.toExtractor());
    }
    return builder.build();
  }

  protected final boolean isFunctor(ImmutableList<Extractor> childExtractors) {
    boolean isFunctor = true;
    for (Extractor extractor : childExtractors) {
      isFunctor = isFunctor && (extractor instanceof Extractor.Functor);
    }
    return isFunctor;
  }

  protected final <T> Optional<T> getOptionalParam(List<Object> args, int index) {
    Preconditions.checkArgument(index >= 0);
    if (index < args.size()) {
      return Optional.fromNullable((T) args.get(index));
    }
    return Optional.<T>absent();
  }

  private void buildGenericTypeIdToTypeMapping(
      Type genericType, Type concreteType, Map<Long, Type> recordMap) throws SemanticCheckFailure {
    if (genericType.isGenericType()) {
      long genericTypeId = genericType.genericTypeId;
      if (!recordMap.containsKey(genericTypeId)) {
        recordMap.put(genericTypeId, concreteType);
      } else {
        recordMap.put(
            genericTypeId,
            Type.getClosestCommonSuperType(
                concreteType,
                recordMap.get(genericTypeId)
            )
        );
      }
    } else {
      if (Type.isDivergentTo(genericType.typeBase, concreteType.typeBase)) {
        throw new SemanticCheckFailure(String.format(
            "type checking expression %s failed: invalid argument type: %s received, %s expected",
            exprText,
            concreteType.toString(),
            genericType.toString()));
      }
      ImmutableList<Type> genericTypeParams = genericType.getTypeParams();
      ImmutableList<Type> concreteTypeParams = concreteType.getTypeParams();
      if (genericTypeParams.size() == concreteTypeParams.size()) {
        for (int i = 0; i < genericTypeParams.size(); i++) {
          buildGenericTypeIdToTypeMapping(
              genericTypeParams.get(i),
              concreteTypeParams.get(i),
              recordMap
          );
        }
      }
    }
  }

  private void validateSignature() throws SemanticCheckFailure {
    int minArgs = signature.paramTypes.size();
    int maxArgs = signature.paramTypes.size() + signature.optParamTypes.size();

    if (signature.varargsType == null) {
      if (astNodeChildren.size() < minArgs
          || astNodeChildren.size() > maxArgs) {
        if (minArgs == maxArgs) {
          throw new SemanticCheckFailure(String.format(
              "type checking expression %s failed: expects %d arguments, %d passed",
              exprText,
              minArgs,
              astNodeChildren.size()
          ));
        } else {
          throw new SemanticCheckFailure(String.format(
              "type checking expression %s failed: expects %d arguments to %d arguments, %d passed",
              exprText,
              minArgs,
              maxArgs,
              astNodeChildren.size()
          ));
        }
      }
    } else {
      if (astNodeChildren.size() < minArgs) {
        throw new SemanticCheckFailure(String.format(
            "type checking expression %s failed: expects at least %d arguments, %d passed",
            exprText,
            minArgs,
            astNodeChildren.size()
        ));
      }
    }
  }

  private Type inferReturnTypeAndDoTypeChecking() throws SemanticCheckFailure {
    ImmutableList<Type> paramTypes = signature.paramTypes;
    ImmutableList<Type> optParamTypes = signature.optParamTypes;
    Type varargsType = signature.varargsType;
    Type sigReturnType = signature.returnType;

    List<ASTNode> passedParams = getChildren();

    Map<Long, Type> mappedTypes = Maps.newHashMap();

    for (int i = 0; i < paramTypes.size(); i++) {
      buildGenericTypeIdToTypeMapping(
          paramTypes.get(i),
          passedParams.get(i).getReturnType(),
          mappedTypes
      );
    }

    int optArgsStartPos = paramTypes.size();
    int varargStartPos = paramTypes.size() + optParamTypes.size();
    int passedOptParamEnds = Math.min(varargStartPos, passedParams.size());
    for (int i = optArgsStartPos; i < passedOptParamEnds; i++) {
      buildGenericTypeIdToTypeMapping(
          optParamTypes.get(i - optArgsStartPos),
          passedParams.get(i).getReturnType(),
          mappedTypes
      );
    }

    for (int i = varargStartPos; i < passedParams.size(); i++) {
      buildGenericTypeIdToTypeMapping(
          varargsType,
          passedParams.get(i).getReturnType(),
          mappedTypes
      );
    }

    return sigReturnType.replaceTypeArguments(mappedTypes);
  }

  public void visit(PartialFunction<ASTNode, Unit> visitor) {
    if (visitor.isDefinedAt(this)) {
      visitor.apply(this);
    }

    for (ASTNode child : this.getChildren()) {
      if (child != null) {
        child.visit(visitor);
      }
    }
  }

  @Override
  public int hashCode() {
    return getFingerprint().hashCode();
  }

  @Override
  public boolean equals(Object object) {
    if (this == object) {
      return true;
    }
    if (!(object instanceof ASTNode)) {
      return false;
    }
    ASTNode that = (ASTNode) object;
    return Objects.equals(this.getFingerprint(), that.getFingerprint());
  }

  @Override
  public String toString() {
    return this.exprText + ", "
        + this.className + ", "
        + this.astNodeChildren.size();
  }

  public enum CacheLevel {
    Min(-1000),
    Global(-100),
    Event(-10),
    Rule(-1),
    ConstantContextScope(0),
    Never(Long.MAX_VALUE - 1000),
    Max(Long.MAX_VALUE);

    CacheLevel(long level) {
      this.scope = level;
    }

    public final long scope;
  }

  public static class Signature {
    public final ImmutableList<Type> paramTypes;
    public final ImmutableList<Type> optParamTypes;
    public final Type varargsType;
    public final Type returnType;
    public final Fingerprint fingerprint;

    public Signature(Type varargsType, Type returnType) {
      this(ImmutableList.of(), ImmutableList.of(), varargsType, returnType);
    }

    public Signature(ImmutableList<Type> paramTypes, Type returnType) {
      this(paramTypes, ImmutableList.<Type>of(), null, returnType);
    }

    public Signature(ImmutableList<Type> paramTypes, Type varargsType, Type returnType) {
      this(paramTypes, ImmutableList.<Type>of(), varargsType, returnType);
    }

    public Signature(
        ImmutableList<Type> paramTypes,
        ImmutableList<Type> optParamTypes,
        Type returnType) {
      this(paramTypes, optParamTypes, null, returnType);
    }

    private Signature(
        ImmutableList<Type> paramTypes,
        ImmutableList<Type> optParamTypes,
        Type varargsType,
        Type returnType) {
      this.paramTypes = paramTypes;
      this.optParamTypes = optParamTypes;
      this.varargsType = varargsType;
      this.returnType = returnType;
      Hasher hasher = Fingerprint.newHasher();
      computerFingerprint(hasher);
      this.fingerprint = new Fingerprint(0, hasher.hash().asBytes());
    }

    public final void computerFingerprint(Hasher hasher) {

      hasher.putInt(paramTypes.size());
      for (Type type : paramTypes) {
        type.computerFingerprint(hasher);
      }

      hasher.putInt(optParamTypes.size());
      for (Type type : optParamTypes) {
        type.computerFingerprint(hasher);
      }

      if (varargsType != null) {
        hasher.putInt(2);
        varargsType.computerFingerprint(hasher);
      }

      hasher.putInt(3);
      returnType.computerFingerprint(hasher);
    }

    @Override
    public boolean equals(Object object) {
      if (this == object) {
        return true;
      }
      if (!(object instanceof Signature)) {
        return false;
      }
      Signature that = (Signature) object;
      return Objects.equals(this.fingerprint, that.fingerprint);
    }

    @Override
    public int hashCode() {
      return fingerprint.hashCode();
    }
  }
}
