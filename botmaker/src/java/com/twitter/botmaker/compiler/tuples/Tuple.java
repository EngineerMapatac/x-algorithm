package com.twitter.botmaker.compiler.tuples;

import java.util.Objects;

import org.apache.commons.lang3.builder.HashCodeBuilder;

public interface Tuple {

  int size();

  Object get(int index);

  static boolean equals(Tuple tuple, Object o) {
    if (o == tuple) {
      return true;
    }

    if (!(o instanceof Tuple)) {
      return false;
    }

    Tuple that = (Tuple) o;
    if (tuple.size() != that.size()) {
      return false;
    }

    for (int i = 0; i < tuple.size(); i++) {
      if (!Objects.equals(tuple.get(i), that.get(i))) {
        return false;
      }
    }

    return true;

  }

  static String toString(Tuple tuple) {
    StringBuilder sb = new StringBuilder();
    sb.append('(');

    if (tuple.size() > 0) {
      for (int index = 0; index < tuple.size() - 1; index++) {
        sb.append(tuple.get(index));
        sb.append(", ");
      }
      sb.append(tuple.get(tuple.size() - 1));
    }

    sb.append(')');
    return sb.toString();
  }

  static int hashCode(Tuple tuple) {
    HashCodeBuilder hasher = new HashCodeBuilder();
    for (int i = 0; i < tuple.size(); i++) {
      hasher.append(tuple.get(i));
    }

    return hasher.toHashCode();
  }

  static <T1, T2> Pair<T1, T2> of(T1 t1, T2 t2) {
    return new Pair<>(t1, t2);
  }

  static <T1, T2, T3> Tuple3<T1, T2, T3> of(T1 t1, T2 t2, T3 t3) {
    return new Tuple3<>(t1, t2, t3);
  }

  static <T1, T2, T3, T4> Tuple4<T1, T2, T3, T4> of(T1 t1, T2 t2, T3 t3, T4 t4) {
    return new Tuple4<>(t1, t2, t3, t4);
  }

  static Tuple of(Object[] values) {
    if (values.length == 2) {
      return of(values[0], values[1]);
    } else if (values.length == 3) {
      return of(values[0], values[1], values[2]);
    } else if (values.length == 4) {
      return of(values[0], values[1], values[2], values[3]);
    } else {
      return new Tuple() {

        @Override
        public int size() {
          return values.length;
        }

        @Override
        public Object get(int index) {
          return values[index];
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
      };
    }
  }
}
