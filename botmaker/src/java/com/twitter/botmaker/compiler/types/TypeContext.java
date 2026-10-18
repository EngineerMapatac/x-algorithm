package com.twitter.botmaker.compiler.types;

import com.google.common.base.Optional;

public interface TypeContext {

  TypeContext EMPTY = new TypeContext() {

    @Override
    public Optional<Type> getTypeFromName(String typeName) {
      return Optional.absent();
    }
  };

  Optional<Type> getTypeFromName(String typeName);
}
