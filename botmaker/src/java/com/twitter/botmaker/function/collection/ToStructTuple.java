package com.twitter.botmaker.function.collection;

import java.util.List;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.tuples.StructTuple;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.Constant;
import com.twitter.botmaker.function.FunctionNode;
import com.twitter.botmaker.function.conversion.ToPair;

public abstract class ToStructTuple extends FunctionNode<Runtime> {

  public final ImmutableList<String> fieldNames;

  public ToStructTuple(
      String exprText,
      ImmutableList<String> fieldNames,
      List<ASTNode> children
  ) throws SemanticCheckFailure {
    super(exprText, children);
    this.fieldNames = fieldNames;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Global;
  }

  @Override
  protected Object apply(Context<Runtime> context, List<Object> args) throws Exception {
    return StructTuple.of(fieldNames, args);
  }

  public static ToStructTuple of(
      String funcName, ImmutableList<ASTNode> children) throws SemanticCheckFailure {

    if (children.size() == 0) {
      throw new SemanticCheckFailure("StructTuple should have at least one field");
    }

    ImmutableList.Builder<String> fieldNamesBuilder = ImmutableList.builder();
    ImmutableList.Builder<Type> fieldTypesBuilder = ImmutableList.builder();
    ImmutableList.Builder<ASTNode> childrenBuilder = ImmutableList.builder();
    for (ASTNode child : children) {
      if (!(child instanceof ToPair)) {
        throw new SemanticCheckFailure(
            String.format(
                "StructTuple expects pairs of name and values, but a %s received", child));
      }

      fieldNamesBuilder.add(((Constant) child.getChildren().get(0)).getValue().toString());
      fieldTypesBuilder.add(((ASTNode) child.getChildren().get(1)).getReturnType());
      childrenBuilder.add((ASTNode) child.getChildren().get(1));
    }

    ImmutableList<String> fieldNames = fieldNamesBuilder.build();
    ImmutableList<Type> fieldTypes = fieldTypesBuilder.build();
    Signature signature = new Signature(
        fieldTypes,
        Type.structTupleOf(fieldNames, fieldTypes)
    );

    return new ToStructTuple(funcName, fieldNames, childrenBuilder.build()) {

      @Override
      public Signature getSignature() {
        return signature;
      }
    };
  }
}
