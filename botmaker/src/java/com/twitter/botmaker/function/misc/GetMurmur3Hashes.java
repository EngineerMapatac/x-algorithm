package com.twitter.botmaker.function.misc;

import java.util.List;

import com.google.common.collect.ImmutableList;
import com.google.common.collect.Lists;
import com.google.common.hash.HashFunction;
import com.google.common.hash.Hashing;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    argTypes = {"Long", "List<String>"},
    arguments = {"seed", "a list of strings"},
    returnType = "List<Long>",
    description = "Returns the murmur3 hashes for the given strings.",
    actionLevel = ActionLevel.NO_ACTION
)
public class GetMurmur3Hashes extends FunctionNode2<Runtime, Long, List<String>> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.LONG, Type.listOf(Type.STRING)),
      Type.listOf(Type.LONG)
  );

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  public GetMurmur3Hashes(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected Object apply(Context<Runtime> context, Long seed, List<String> strings) {
    HashFunction hashFunction = Hashing.murmur3_128(seed.intValue());
    List<Long> hashes = Lists.newArrayListWithCapacity(strings.size());
    for (String string : strings) {
      hashes.add(hashFunction.hashUnencodedChars(string).asLong());
    }
    return hashes;
  }
}
