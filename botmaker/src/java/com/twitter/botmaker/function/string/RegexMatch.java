package com.twitter.botmaker.function.string;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.RegexCache;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

public class RegexMatch extends FunctionNode2<Runtime, String, String> {

  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      Type.BOOLEAN
  );

  public RegexMatch(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
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
  public Object apply(Context<Runtime> context, String str, String regex) {

    final RegexCache regexCache;
    if (context.getRuntime() instanceof RegexCache) {
      regexCache = (RegexCache) (context.getRuntime());
    } else {
      regexCache = RegexCache.DEFAULT;
    }

    return regexCache.matches(context, regex, str);
  }
}
