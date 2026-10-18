package com.twitter.botmaker.function.string;

import java.math.BigInteger;

import com.google.common.base.Preconditions;
import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;

@BotMakerFunction(
    name = {"CosineDist"},
    argTypes = {"String", "String"},
    arguments = {"hex string 1 for cosine distance", "hex string 2 for cosine distance"},
    returnType = "Double",
    description = "Calculate the cosine distance between two hex strings",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {"CosineDist(\"ab\", \"ac\")"}
)
public class CosineDist extends FunctionNode2<Runtime, String, String> {
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      Type.DOUBLE
  );

  public CosineDist(String exprText, ImmutableList<ASTNode> children) throws SemanticCheckFailure {
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
  protected Double apply(Context<Runtime> context, String string1, String string2) {
    return cosineDist(string1, string2);
  }

  static double cosineDist(String x, String y) throws ArithmeticException, NumberFormatException {
    Preconditions.checkArgument(x.length() == y.length(),
        "two strings must be of the same length");

    BigInteger num1 = new BigInteger('+' + x, 16);
    BigInteger num2 = new BigInteger('+' + y, 16);

    long dotProduct = HammingDist.hammingDist(num1.and(num2), BigInteger.ZERO);
    long num1Magnitude = HammingDist.hammingDist(num1, BigInteger.ZERO);
    long num2Magnitude = HammingDist.hammingDist(num2, BigInteger.ZERO);
    return 1 - (dotProduct / Math.sqrt(Math.multiplyExact(num1Magnitude, num2Magnitude)));
  }
}
