package com.twitter.botmaker.function.conversion;

import java.util.Map;

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

public abstract class ThriftFromDict extends FunctionNode1<Runtime, Map<Object, Object>> {

  private ThriftFromDict(
      String exprText,
      ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {
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
          ImmutableList.of(Type.mapOf(Type.OBJECT, Type.OBJECT)),
          thriftType
      );

      return new ThriftFromDict(exprText, ImmutableList.of(node)) {

        @Override
        protected Object apply(
            Context<Runtime> context,
            Map<Object, Object> arg
        ) throws Exception {
          return thriftType.newInstance(arg);
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
