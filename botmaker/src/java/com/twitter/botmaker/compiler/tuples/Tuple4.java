package com.twitter.botmaker.compiler.tuples;

public class Tuple4<T1, T2, T3, T4> implements Tuple {

  private final T1 t1;
  private final T2 t2;
  private final T3 t3;
  private final T4 t4;

  public Tuple4(T1 t1, T2 t2, T3 t3, T4 t4) {
    this.t1 = t1;
    this.t2 = t2;
    this.t3 = t3;
    this.t4 = t4;
  }

  @Override
  public int size() {
    return 4;
  }

  @Override
  public Object get(int index) {
    if (index == 0) {
      return t1;
    } else if (index == 1) {
      return t2;
    } else if (index == 2) {
      return t3;
    } else if (index == 3) {
      return t4;
    } else {
      throw new IndexOutOfBoundsException();
    }
  }

  public T1 getFirst() {
    return t1;
  }

  public T2 getSecond() {
    return t2;
  }

  public T3 getThird() {
    return t3;
  }

  public T4 getFourth() {
    return t4;
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
