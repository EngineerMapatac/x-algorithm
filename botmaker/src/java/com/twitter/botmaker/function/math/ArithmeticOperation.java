package com.twitter.botmaker.function.math;

import com.google.common.collect.ImmutableList;
import com.google.common.math.DoubleMath;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode2;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.finagle.stats.DefaultStatsReceiver;
import com.twitter.finagle.stats.StatsReceiver;
import com.twitter.util.Function3;

public abstract class ArithmeticOperation<T> extends FunctionNode2<Runtime, T, T> {

  private static final Signature LONG_SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER, Type.NUMBER),
      Type.LONG
  );

  private static final Signature DOUBLE_SIGNATURE = new Signature(
      ImmutableList.of(Type.NUMBER, Type.NUMBER),
      Type.DOUBLE
  );

  private static final Signature STRING_SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING),
      Type.STRING
  );

  private static final StatsReceiver DEFAULT_STATS_RECEIVER = DefaultStatsReceiver.scope(
      "botmaker", "function", "math", "ArithmeticOperation");

  public ArithmeticOperation(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  private static final Function3<Context, Number, Number, Number> ADD_FUNCTION =
      new Function3<Context, Number, Number, Number>() {
        @Override
        public Number apply(Context context, Number numA, Number numB) {
          if (Type.longLike(numA) && Type.longLike(numB)) {
            return numA.longValue() + numB.longValue();
          }
          return numA.doubleValue() + numB.doubleValue();
        }
      };

  private static final Function3<Context, Number, Number, Number> SUB_FUNCTION =
      new Function3<Context, Number, Number, Number>() {
        @Override
        public Number apply(Context context, Number numA, Number numB) {
          if (Type.longLike(numA) && Type.longLike(numB)) {
            return numA.longValue() - numB.longValue();
          }
          return numA.doubleValue() - numB.doubleValue();
        }
      };

  private static final Function3<Context, Number, Number, Number> MUL_FUNCTION =
      new Function3<Context, Number, Number, Number>() {
        @Override
        public Number apply(Context context, Number numA, Number numB) {
          if (Type.longLike(numA) && Type.longLike(numB)) {
            return numA.longValue() * numB.longValue();
          }
          return numA.doubleValue() * numB.doubleValue();
        }
      };

  private static final Function3<Context, Number, Number, Number> DIV_FUNCTION =
      new Function3<Context, Number, Number, Number>() {
        @Override
        public Number apply(Context context, Number numA, Number numB) {
          double tolerance = 0.000001d;
          if (DoubleMath.fuzzyEquals(numB.doubleValue(), 0.0, tolerance)) {
            if (DoubleMath.fuzzyEquals(numA.doubleValue(), 0.0, tolerance)) {
              DEFAULT_STATS_RECEIVER.counter(
                  "zero_divide_by_zero_calls", String.valueOf(context.getTrackingId()))
                  .incr();
            }
            else {
              DEFAULT_STATS_RECEIVER.counter(
                  "illegal_divide_by_zero_calls", String.valueOf(context.getTrackingId()))
                  .incr();
            }
          }
          return numA.doubleValue() / numB.doubleValue();
        }
      };

  private static final Function3<Context, Number, Number, Number> DIV_SAFE_FUNCTION =
      new Function3<Context, Number, Number, Number>() {
        @Override
        public Number apply(Context context, Number numA, Number numB) {
          if (numB.doubleValue() == 0.0) {
            return 0.0;
          }
          return numA.doubleValue() / numB.doubleValue();
        }
      };

  private static final Function3<Context, String, String, String> CONCAT_FUNCTION =
      new Function3<Context, String, String, String>() {
        @Override
        public String apply(Context context, String strA, String strB) {
          StringBuilder stringBuilder = new StringBuilder();
          stringBuilder.append(strA);
          stringBuilder.append(strB);
          return stringBuilder.toString();
        }
      };

  private static <T> ArithmeticOperation of(
      String exprText,
      ImmutableList<ASTNode> children,
      Signature signature,
      final Function3<Context, T, T, T> function) throws SemanticCheckFailure {
    return new ArithmeticOperation<T>(exprText, children) {
      @Override
      public Signature getSignature() {
        return signature;
      }

      @Override
      protected Object apply(Context<Runtime> context, T numA, T numB) {
        return function.apply(context, numA, numB);
      }
    };
  }

  private static void validateSignature(
      String exprText,
      ImmutableList<ASTNode> children
  ) throws SemanticCheckFailure {
    if (children.size() != 2) {
      throw new SemanticCheckFailure(String.format(
          "type checking expression %s failed: expects 2 arguments, %d passed",
          exprText,
          children.size()
      ));
    }
  }

  public static ArithmeticOperation ofSum(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    validateSignature(exprText, children);
    if (children.get(0).getReturnType().longLike()
        && children.get(1).getReturnType().longLike()) {
      return of(exprText, children, LONG_SIGNATURE, ADD_FUNCTION);
    } else if (children.get(0).getReturnType().stringLike()
        && children.get(1).getReturnType().stringLike()) {
      return of(exprText, children, STRING_SIGNATURE, CONCAT_FUNCTION);
    } else {
      return of(exprText, children, DOUBLE_SIGNATURE, ADD_FUNCTION);
    }
  }

  public static ArithmeticOperation ofDifference(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    if (children.get(0).getReturnType().longLike() && children.get(1).getReturnType().longLike()) {
      return of(exprText, children, LONG_SIGNATURE, SUB_FUNCTION);
    } else {
      return of(exprText, children, DOUBLE_SIGNATURE, SUB_FUNCTION);
    }
  }

  public static ArithmeticOperation ofProduct(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    if (children.get(0).getReturnType().longLike() && children.get(1).getReturnType().longLike()) {
      return of(exprText, children, LONG_SIGNATURE, MUL_FUNCTION);
    } else {
      return of(exprText, children, DOUBLE_SIGNATURE, MUL_FUNCTION);
    }
  }

  public static ArithmeticOperation ofQuotient(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    return of(exprText, children, DOUBLE_SIGNATURE, DIV_FUNCTION);
  }

  public static ArithmeticOperation ofQuotientSafe(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    return of(exprText, children, DOUBLE_SIGNATURE, DIV_SAFE_FUNCTION);
  }
}
