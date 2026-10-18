package com.twitter.botmaker.function;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.Pair;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

public class Val extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(ImmutableList.of(TP), Type.UNIT);

  private final String varName;
  private final ASTNode varExprNode;
  private final Type varType;

  public Val(String exprText, Pair<String, ASTNode> assignment)
      throws SemanticCheckFailure {
    super(exprText, ImmutableList.of(assignment.getSecond()));
    this.varName = intern(assignment.getFirst());
    this.varExprNode = assignment.getSecond();
    this.varType = varExprNode.getReturnType();
  }

  public String getVariableName() {
    return varName;
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
  public Extractor<Runtime> toExtractor() {

    final Extractor varExprExtractor = varExprNode.toExtractor();
    final ImmutableList<Extractor> childExtractors = ImmutableList.of(varExprExtractor);

    return new Functor<Runtime>(this, childExtractors) {

      @Override
      public Future run(Context<Runtime> context) {
        return Future.apply(Function.exfunc0(() -> apply(context)));
      }

      @Override
      public Object apply(Context<Runtime> context) throws Exception {
        Future value = context.run(varExprExtractor);
        context.newVariable(varName, value);
        return unit();
      }
    };
  }
}
