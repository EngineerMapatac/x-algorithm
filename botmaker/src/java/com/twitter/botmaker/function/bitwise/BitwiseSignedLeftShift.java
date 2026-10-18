package com.twitter.botmaker.function.bitwise;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;

@BotMakerFunction(
    argTypes = {
        "Long",
        "Long"
    },
    arguments = {
        "A",
        "B"
    },
    returnType = "Long",
    description = "Returns SIGNED bitwise (A << B).",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "BitwiseSignedLeftShift(1, 0)"
    }
)
public class BitwiseSignedLeftShift extends BitwiseOp {
  public BitwiseSignedLeftShift(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
  }

  @Override
  Long doTheMath(Long a, Long b) {
    return a << b;
  }
}
