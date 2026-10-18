package com.twitter.botmaker.config;

import java.util.Map;

import scala.Tuple3;

import com.twitter.botmaker.compiler.types.Type;
import com.twitter.botmaker.runtime.Runtime;

public interface ConfigContainer extends Runtime {

  void addConfig(String configName, Type configType, String description, Object configValue);

  static ConfigContainer of(Map<String, Tuple3<Type, String, Object>> container) {

    return new ConfigContainer() {
      @Override
      public void addConfig(
          String configName, Type configType, String description, Object configValue) {
        container.put(
            configName,
            Tuple3.apply(configType, description, configValue)
        );
      }
    };
  }
}
