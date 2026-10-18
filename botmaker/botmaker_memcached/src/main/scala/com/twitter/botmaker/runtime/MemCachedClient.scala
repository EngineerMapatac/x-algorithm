package com.twitter.botmaker.runtime

import java.nio.ByteBuffer
import java.util
import scala.collection.JavaConverters._
import scala.collection.convert.WrapAsJava
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.thriftjava.MemCachedClientConfig
import com.twitter.io.Buf
import com.twitter.util._

case class MemCachedClientOperator(config: MemCachedClientConfig) {

  private val ns = config.endpoint + "."
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName

  val name: String = config.funcName

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val keyTypes = config.keyTypes.asScala map { a => TypeCompiler.getType(a, typeContext) } toList

    val valueType = TypeCompiler.getType(config.valueType, typeContext)

    val valueSerializer = valueType.serializer

    def makeKey(args: util.List[AnyRef]) = {
      val hasher = Fingerprint.newHasher
      hasher.putUnencodedChars(classOf[MemCachedClientOperator].getName)
      hasher.putUnencodedChars(config.dataset)
      keyTypes.zipWithIndex foreach {
        case (t, i) =>
          t.serializer.computeFingerprint(hasher, args.get(i))
      }
      val bytes = hasher.hash().asBytes()
      java.util.Base64.getUrlEncoder.encodeToString(bytes)
    }

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes :+ valueType)),
        ImmutableList.of(Type.LONG),
        Type.UNIT
      )

      @Override
      def funcName = setFuncName

      override def botMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"set value by key in Memcache. ${config.description}",
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[MemCachedEndpointContainer](setFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[MemCachedEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val key = makeKey(args)
            val buffer = valueSerializer.serialize(args.get(keyTypes.size), true)
            val buf = Buf.ByteBuffer.Owned(buffer)
            val expiry = if (args.size() > keyTypes.size + 1) {
              val ttl = args.get(keyTypes.size + 1).asInstanceOf[java.lang.Long]
              Time.fromMilliseconds(context.getRuntime.clock.nowMillis + ttl)
            } else {
              Time.epoch
            }
            val endpoint = context.getRuntime.memCachedEndpoints(config.endpoint)
            endpoint.set(key, 0, expiry, buf).voided
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes)),
        ImmutableList.of(valueType),
        valueType
      )

      @Override
      def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"get value by key in Memcache. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[MemCachedEndpointContainer](getFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[MemCachedEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val key = makeKey(args)
            val endpoint = context.getRuntime.memCachedEndpoints(config.endpoint)
            endpoint.get(key) map {
              case Some(buf) =>
                val buffer = ByteBuffer.allocate(buf.length)
                buf.write(buffer)
                buffer.flip()
                valueSerializer.deserialize(buffer, true)
              case None if args.size() > keyTypes.size =>
                args.get(keyTypes.size)
              case None =>
                throw new NoSuchElementException
            }
          }
        }
      }
    }

    setFunc :: getFunc :: Nil
  }
}
