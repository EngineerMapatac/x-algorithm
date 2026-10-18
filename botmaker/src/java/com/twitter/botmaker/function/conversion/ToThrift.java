package com.twitter.botmaker.function.conversion;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.Hasher;

import org.apache.thrift.TBase;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ClassCache;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.ThriftType;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;

public abstract class ToThrift extends FunctionNode1<Runtime, Object> {

  private ToThrift(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
    super(exprText, children);
  }

  public static ASTNode mkNewThriftObject(
      String exprText,
      String className,
      ASTNode node
  ) throws SemanticCheckFailure {
    try {
      Class<? extends TBase> classz = (Class<? extends TBase>) ClassCache.forName(className);
      ThriftType thriftType = Type.thriftOf(classz);
      return mkNewThriftObject(exprText, thriftType, node);
    } catch (Exception e) {
      throw new SemanticCheckFailure(e);
    }
  }

  private static ASTNode mkNewThriftObject(
      String exprText,
      ThriftType thriftType,
      ASTNode node
  ) throws SemanticCheckFailure {
    try {
      final Signature signature = new Signature(
          ImmutableList.of(Type.OBJECT),
          thriftType
      );

      return new ToThrift(exprText, ImmutableList.of(node)) {

        @Override
        protected Object apply(
            Context<Runtime> context,
            Object arg
        ) {
          return arg;
        }

        @Override
        public Signature getSignature() {
          return signature;
        }

        @Override
        protected CacheLevel getCacheLevel() {
          return CacheLevel.Global;
        }

        @Override
        protected void computeFingerprint(Hasher hasher) {
          super.computeFingerprint(hasher);
          thriftType.computerFingerprint(hasher);
        }
      };
    } catch (Exception e) {
      throw new SemanticCheckFailure(e);
    }
  }
}
