package com.twitter.botmaker.compiler.tuples;

public class Pair<T1, T2> implements Tuple {

  private final T1 t1;
  private final T2 t2;

  public Pair(T1 t1, T2 t2) {
    this.t1 = t1;
    this.t2 = t2;
  }

  @Override
  public int size() {
    return 2;
  }

  @Override
  public Object get(int index) {
    if (index == 0) {
      return t1;
    } else if (index == 1) {
      return t2;
    } else {
      throw new IndexOutOfBoundsException();
    }
  }

  public static <T1, T2> Pair<T1, T2> of(T1 t1, T2 t2) {
    return new Pair<>(t1, t2);
  }

  public T1 getFirst() {
    return t1;
  }

  public T2 getSecond() {
    return t2;
  }

  @Override
  public boolean equals(Object o) {
    return Tuple.equals(this, o);
  }

  @Override
  public String toString() {
    return Tuple.toString(this);

  }

  @Override
  public int hashCode() {
    return Tuple.hashCode(this);
  }
}
