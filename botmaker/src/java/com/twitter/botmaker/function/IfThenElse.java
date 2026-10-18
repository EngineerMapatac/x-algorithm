package com.twitter.botmaker.function;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.Extractor;
import com.twitter.botmaker.Extractor.Functor;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;
import com.twitter.util.Function;
import com.twitter.util.Future;

@BotMakerFunction(
    name = {"If"},
    argTypes = {
        "Boolean",
        "T",
        "[T]"
    },
    arguments = {
        "condition",
        "then-expression",
        "else-expression"
    },
    returnType = "T",
    description = "If the first expression is true, return the value of the second expression;"
        + " otherwise, return the value of the third expression. The return type is inferred from"
        + " the second and the third expressions.",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "if (1 != 1) { None(); }",
        "if (TRUE) {1} else {2} + 3",
        "\"Number\" == TypeName( if( TRUE ) 1 else 0.1)",
        "\"Object\" == TypeName( if( TRUE ) 1 else \"abc\")",
        "\"List<Object>\" == TypeName( if( TRUE ) [1,2,3] else [\"abc\"])"
    }
)
public class IfThenElse extends ASTNode<Runtime> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.BOOLEAN, TP),
      ImmutableList.of(TP),
      TP
  );

  public IfThenElse(String exprText, ImmutableList<ASTNode> children)
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
  public Extractor<Runtime> toExtractor() {
    final ImmutableList<Extractor> childExtractors = buildExtractorsOfChildren();
    final Extractor condExpr = childExtractors.get(0);
    final Extractor thenExpr = childExtractors.get(1);
    final Extractor elseExpr;
    if (childExtractors.size() > 2) {
      elseExpr = childExtractors.get(2);
    } else {
      elseExpr = null;
    }

    if (isFunctor(childExtractors)) {
      return new Functor<Runtime>(this, childExtractors) {

        @Override
        public Future run(Context<Runtime> context) {
          return IfThenElse.this.run(context, condExpr, thenExpr, elseExpr);
        }

        @Override
        public Object apply(Context<Runtime> context) throws Exception {
          Boolean cond = (Boolean) ((Functor) condExpr).apply(context);
          if (cond) {
            return ((Functor) thenExpr).apply(context);
          } else if (elseExpr != null) {
            return ((Functor) elseExpr).apply(context);
          } else {
            return null;
          }
        }
      };
    } else {
      return new Extractor<Runtime>(this, childExtractors) {

        @Override
        public Future run(Context<Runtime> context) {
          return IfThenElse.this.run(context, condExpr, thenExpr, elseExpr);
        }
      };
    }
  }

  private Future run(
      Context<Runtime> context, Extractor condExpr, Extractor thenExpr, Extractor elseExpr) {
    return context.run(condExpr).flatMap(Function.func(condObj -> {
      if ((Boolean) condObj) {
        return context.run(thenExpr);
      } else if (elseExpr != null) {
        return context.run(elseExpr);
      } else {
        return Future.value(null);
      }
    }));
  }
}
