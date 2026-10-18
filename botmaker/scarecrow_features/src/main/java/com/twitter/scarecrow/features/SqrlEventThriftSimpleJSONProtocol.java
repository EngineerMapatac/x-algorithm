package com.twitter.scarecrow.features;

import java.nio.ByteBuffer;
import java.util.Base64;

import org.apache.thrift.TException;
import org.apache.thrift.protocol.TProtocol;
import org.apache.thrift.protocol.TProtocolFactory;
import org.apache.thrift.protocol.TSimpleJSONProtocol;
import org.apache.thrift.transport.TTransport;

import com.twitter.io.Buf$ByteArray$Owned$;
import com.twitter.io.Buf$ByteBuffer$Owned$;


public class SqrlEventThriftSimpleJSONProtocol extends TSimpleJSONProtocol {

  public SqrlEventThriftSimpleJSONProtocol(TTransport trans) {
    super(trans);
  }

  public static class Factory implements TProtocolFactory {
    public TProtocol getProtocol(TTransport trans) {
      return new SqrlEventThriftSimpleJSONProtocol(trans);
    }
  }

  @Override
  public void writeBinary(ByteBuffer bin) throws TException {
    writeString(Base64.getEncoder().encodeToString(
        Buf$ByteArray$Owned$.MODULE$.extract(Buf$ByteBuffer$Owned$.MODULE$.apply(bin))));
  }
}
