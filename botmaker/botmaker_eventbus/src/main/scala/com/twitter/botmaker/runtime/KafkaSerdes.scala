package com.twitter.botmaker.runtime

import org.apache.kafka.common.serialization._
import com.twitter.botmaker.compiler.ClassCache
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.finagle.thrift.Protocols
import com.twitter.scrooge.TReusableBuffer
import com.twitter.util.BinaryThriftSerializer

trait KafkaSerdes {

  def getKeySerializer(keyType: String): Serializer[AnyRef] = {
    keyType match {
      case "Long" => new LongSerializer().asInstanceOf[Serializer[AnyRef]]
      case "Double" => new DoubleSerializer().asInstanceOf[Serializer[AnyRef]]
      case "String" => new StringSerializer().asInstanceOf[Serializer[AnyRef]]
      case "" =>
        new Serializer[AnyRef] {
          override def configure(map: java.util.Map[String, _], b: Boolean): Unit = {}

          override def close(): Unit = {}

          override def serialize(s: String, t: AnyRef): Array[Byte] = null
        }

      case _ => throw new ConfigFailure(s"unknown kafka key type $keyType")
    }
  }

  def getKeyDeserializer(keyType: String): Deserializer[AnyRef] = {
    keyType match {
      case "Long" => new LongDeserializer().asInstanceOf[Deserializer[AnyRef]]
      case "Double" => new DoubleDeserializer().asInstanceOf[Deserializer[AnyRef]]
      case "String" => new StringDeserializer().asInstanceOf[Deserializer[AnyRef]]
      case "" =>
        new Deserializer[AnyRef] {
          override def configure(map: java.util.Map[String, _], b: Boolean): Unit = {}

          override def deserialize(s: String, bytes: Array[Byte]): AnyRef = {
            null
          }

          override def close(): Unit = {}
        }

      case _ => throw new ConfigFailure(s"unknown kafka key type $keyType")
    }
  }

  def getValueSerializer(valueClass: String): Serializer[AnyRef] = {
    val classObject = ClassCache.forName(valueClass)
    if (classOf[Thrift[_, _]].isAssignableFrom(classObject)) {
      new Serializer[AnyRef] {
        override def configure(map: java.util.Map[String, _], b: Boolean): Unit = ()

        override def close(): Unit = ()

        override def serialize(s: String, t: AnyRef): Array[Byte] = {
          KafkaSerdes.serialize(t.asInstanceOf[Thrift[_, _]])
        }
      }
    } else if (classOf[String].isAssignableFrom(classObject)) {
      new StringSerializer().asInstanceOf[Serializer[AnyRef]]
    } else {
      new Serializer[AnyRef] {
        override def configure(map: java.util.Map[String, _], b: Boolean): Unit = {}

        override def close(): Unit = {}

        override def serialize(s: String, t: AnyRef): Array[Byte] = null
      }
    }
  }

  def getValueDeserializer(valueClass: String): Deserializer[AnyRef] = {
    val classObject = ClassCache.forName(valueClass)
    if (classOf[Thrift[_, _]].isAssignableFrom(classObject)) {
      val valueClass = classObject.asInstanceOf[Class[Thrift[_, _]]]
      new Deserializer[AnyRef] {
        override def configure(map: java.util.Map[String, _], b: Boolean): Unit = ()

        override def deserialize(s: String, bytes: Array[Byte]): AnyRef = {
          val ser = new BinaryThriftSerializer()
          val obj = valueClass.newInstance
          ser.fromBytes(obj, bytes)
          obj
        }

        override def close(): Unit = ()
      }
    } else if (classOf[String].isAssignableFrom(classObject)) {
      new StringDeserializer().asInstanceOf[Deserializer[AnyRef]]
    } else {
      new Deserializer[AnyRef] {
        override def configure(map: java.util.Map[String, _], b: Boolean): Unit = {}

        override def deserialize(s: String, bytes: Array[Byte]): AnyRef = {
          null
        }

        override def close(): Unit = {}
      }
    }
  }
}

object KafkaSerdes {
  private[this] val binaryFactory = Protocols.binaryFactory()
  private[this] val buffer = TReusableBuffer()

  def serialize(thrift: Thrift[_, _]): Array[Byte] = {
    val transport = buffer.take()
    try {
      thrift.write(binaryFactory.getProtocol(transport))
      java.util.Arrays.copyOf(transport.getArray(), transport.numWrittenBytes)
    } finally {
      transport.reset()
    }
  }

}
