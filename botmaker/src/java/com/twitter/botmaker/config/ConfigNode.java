package com.twitter.botmaker.config;

import com.google.common.collect.ImmutableList;

import com.twitter.botmaker.ASTNode;
import com.twitter.botmaker.Context;
import com.twitter.botmaker.compiler.ActionLevel;
import com.twitter.botmaker.compiler.BotMakerFunction;
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure;
import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.function.FunctionNode3;

@BotMakerFunction(
    name = {"Config"},
    argTypes = {"String", "String", "Object"},
    arguments = {"Config name", "Config description", "Config value"},
    returnType = "Object",
    description = "Update a config value",
    actionLevel = ActionLevel.NO_ACTION,
    examples = {
        "Config(\"TimeoutMills\", \"timeout for service ABC.\", 100)"
    }
)
public class ConfigNode extends FunctionNode3<ConfigContainer, String, String, Object> {

  private static final Type TP = Type.newGenericType();
  private static final Signature SIGNATURE = new Signature(
      ImmutableList.of(Type.STRING, Type.STRING, TP),
      TP
  );

  private final Type configType;

  public ConfigNode(String exprText, ImmutableList<ASTNode> children)
      throws SemanticCheckFailure {
    super(exprText, children);
    configType = children.get(2).getReturnType();
  }

  @Override
  protected Object apply(
      Context<ConfigContainer> context,
      String configName,
      String description,
      Object configValue) throws Exception {
    context.getRuntime().addConfig(configName, configType, description, configValue);
    return configValue;
  }

  @Override
  public Signature getSignature() {
    return SIGNATURE;
  }

  @Override
  protected CacheLevel getCacheLevel() {
    return CacheLevel.Event;
  }
}
