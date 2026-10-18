package com.twitter.botmaker.compiler;

import java.nio.ByteBuffer;

import com.google.common.hash.HashFunction;
import com.google.common.hash.Hasher;
import com.google.common.hash.Hashing;

import org.apache.thrift.TBase;

import com.twitter.botmaker.compiler.types.Type;
import com.twitter.scrooge.ThriftStruct;


public class Fingerprint {

  public final long fp1;
  public final long fp2;
  public final long scope;

  public Fingerprint(long scope, byte[] bytes) {
    ByteBuffer buf = ByteBuffer.wrap(bytes);
    this.fp1 = buf.getLong();
    this.fp2 = buf.getLong();
    this.scope = scope;
  }

  private Fingerprint() {
    this.fp1 = 0L;
    this.fp2 = 0L;
    this.scope = 0L;
  }

  public static final Fingerprint EMPTY = new Fingerprint();

  public void computerFingerprint(Hasher hasher) {
    hasher.putLong(fp1);
    hasher.putLong(fp2);
    hasher.putLong(scope);
  }

  public byte[] getBytes() {
    ByteBuffer buf = ByteBuffer.allocate(16);
    buf.putLong(fp1);
    buf.putLong(fp2);
    return buf.array();
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) {
      return true;
    } else if (o == null) {
      return false;
    } else if (!(o instanceof Fingerprint)) {
      return false;
    } else {
      Fingerprint that = (Fingerprint) o;
      return this.fp1 == that.fp1
          && this.fp2 == that.fp2
          && this.scope == that.scope;
    }
  }

  @Override
  public int hashCode() {
    int result = (int) (fp1 ^ (fp1 >>> 32));
    result = 31 * result + (int) (fp2 ^ (fp2 >>> 32));
    result = 31 * result + (int) (scope ^ (scope >>> 32));
    return result;
  }

  @Override
  public String toString() {
    return "Fingerprint(" + fp1 + "," + fp2 + "," + scope + ')';
  }

  public static Hasher newHasher() {
    HashFunction hashFunction = Hashing.murmur3_128();
    Hasher hasher = hashFunction.newHasher();
    return hasher;
  }

  public static Fingerprint of(TBase<?, ?> thrift) throws Exception {
    return Type.OBJECT.serializer.computeFingerprint(thrift);
  }

  public static Fingerprint of(ThriftStruct thrift) throws Exception {
    return Type.OBJECT.serializer.computeFingerprint(thrift);
  }
}
