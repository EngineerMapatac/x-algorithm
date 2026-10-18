package com.twitter.botmaker.runtime

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.ScribePublisherOperator._
import com.twitter.botmaker.runtime.thriftjava.ScribePublisherConfig
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.finagle.thrift.Protocols
import com.twitter.scrooge.TReusableBuffer
import com.twitter.util.Future

import java.util.{List => JList}
import org.apache.commons.codec.binary.Base64
import org.apache.scribe.LogEntry
import org.apache.scribe.ResultCode
import org.apache.scribe.scribe
import org.apache.thrift.TBase

case class ScribePublisherOperator(config: ScribePublisherConfig) {

  private val thriftClass = ClassCache
    .forName(config.thriftClass)
    .asInstanceOf[Class[TwitterRuntime.Thrift[_, _]]]

  private val ns = "sp."
  private val funcName = ns + config.funcName

  def name: String = config.getFuncName

  def toCompile: CompilerUnit = {

    new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type](Type.thriftOf(thriftClass)),
        Type.UNIT
      )

      @Override
      def funcName = ScribePublisherOperator.this.funcName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        config.description,
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[ThriftEndpointContainer](funcName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            extractor: Extractor[ThriftEndpointContainer],
            context: Context[ThriftEndpointContainer],
            args: JList[AnyRef]
          ): Future[AnyRef] = {
            val message = args.get(0).asInstanceOf[TwitterRuntime.Thrift[_, _]]
            val scribeClient = context.getRuntime
              .thriftEndpoints[scribe.ServiceIface](config.thriftEndpoint)

            Future.Unit.flatMap { _ =>
              val rawMsg = serialize(message)
              val entry = new LogEntry()
              entry.setCategory(config.category)
              entry.setMessage(Base64.encodeBase64String(rawMsg) + '\n')
              scribeClient.Log(ImmutableList.of(entry))
            } onFailure { throwable: Throwable =>
              context.reportFailure("ScribePublisher", funcName, throwable)
            } onSuccess { code: ResultCode =>
              context.reportStat(config.funcName, code.toString, 1L)
            }
          }

          override protected def evaluate(
            context: Context[ThriftEndpointContainer],
            args: JList[AnyRef]
          ): Future[AnyRef] = {
            ???
          }
        }
      }
    }
  }
}

object ScribePublisherOperator {
  private[this] val binaryFactory = Protocols.binaryFactory()
  private[this] val buffer = TReusableBuffer()

  private def serialize(thrift: TBase[_, _]): Array[Byte] = {
    val transport = buffer.take()
    try {
      thrift.write(binaryFactory.getProtocol(transport))
      java.util.Arrays.copyOf(transport.getArray(), transport.numWrittenBytes)
    } finally {
      buffer.reset()
    }
  }

}
