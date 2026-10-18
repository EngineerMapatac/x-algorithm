package com.twitter.botmaker.util;

import java.util.Objects;

@FunctionalInterface
public interface TriConsumer<T, U, V> {

  void accept(T t, U u, V v);

  default TriConsumer<T, U, V> andThen(TriConsumer<? super T, ? super U, ? super V> after) {
    Objects.requireNonNull(after);

    return (first, second, third) -> {
      accept(first, second, third);
      after.accept(first, second, third);
    };
  }
}
