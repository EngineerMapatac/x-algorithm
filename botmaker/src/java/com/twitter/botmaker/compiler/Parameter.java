package com.twitter.botmaker.compiler;

import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;
import com.google.common.hash.Hasher;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;

public class Parameter extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(ImmutableList.of(TP), TP);

  private final String parameterName;

  protected Parameter(
      String exprText,
      ImmutableList<ASTNode> astNodeChildren,
      Type returnType,
      byte[] fingerprintBytes) {
    super(exprText, astNodeChildren, SIGNATURE, returnType, fingerprintBytes);
    this.parameterName = exprText;
  }

  public String getParameterName() {
    return parameterName;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public static Parameter of(String exprText, Type returnType) {

    Hasher hasher = Fingerprint.newHasher();

    hasher.putUnencodedChars(Parameter.class.getName());
    hasher.putUnencodedChars(Strings.nullToEmpty(exprText));
    returnType.computerFingerprint(hasher);

    byte[] fingerprintBytes = hasher.hash().asBytes();
    return new Parameter(exprText, ImmutableList.of(), returnType, fingerprintBytes);
  }

  public static Parameter variable(String exprText, Type returnType, ASTNode parameter) {
    byte[] fingerprintBytes = parameter.getFingerprint().getBytes();
    return new Parameter(exprText, ImmutableList.of(parameter), returnType, fingerprintBytes);
  }

  public static Parameter scope(
      String exprText, Type returnType, long scope, ASTNode collection) {

    Hasher hasher = Fingerprint.newHasher();
    hasher.putUnencodedChars(Parameter.class.getName());
    collection.getFingerprint().computerFingerprint(hasher);

    byte[] fingerprintBytes = hasher.hash().asBytes();
    return new Parameter(exprText, ImmutableList.of(collection), returnType, fingerprintBytes) {

      @Override
      public long computeFingerprintScope(long currentScope) {
        if (scope <= currentScope) {
          return scope;
        } else {
          return super.computeFingerprintScope(currentScope);
        }
      }
    };
  }

  public static Parameter open(String exprText, Type returnType) {
    byte[] fingerprintBytes = Fingerprint.EMPTY.getBytes();
    return new Parameter(exprText, ImmutableList.of(), returnType, fingerprintBytes) {

      @Override
      protected CacheLevel getCacheLevel() {
        return CacheLevel.Never;
      }
    };
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  public Extractor<Runtime> toExtractor() {
    throw new RuntimeException("programming error: Parameter.toExtractor() is called");
  }
}
