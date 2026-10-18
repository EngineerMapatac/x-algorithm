package com.twitter.botmaker.function.misc;

import com.google.common.base.Charsets;
import com.google.common.collect.ImmutableList;
import com.google.common.hash.Hashing;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode1;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"String"},
    arguments = {"string to fingerprint"},
    returnType = "Long",
    description = "Returns the farm fingerprint of the given string."
        + " Equivalent to BigQuery's FARM_FINGERPRINT.",
    actionLevel = ActionLevel.NO_ACTION
)
public class FarmFingerprintAsLong extends FunctionNode1<Runtime, String> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING),
      Type.LONG
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public FarmFingerprintAsLong(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, String input) {
    return Hashing.farmHashFingerprint64().hashString(input, Charsets.UTF_8).asLong();
  }
}
