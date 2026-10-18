package com.twitter.botmaker.runtime

import java.util
import scala.collection.JavaConverters._
import scala.collection.convert.WrapAsJava
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.thriftjava.MemCachedCounterConfig
import com.twitter.io.Buf
import com.twitter.util._

case class MemCachedCounterOperator(config: MemCachedCounterConfig) {

  private val ns = config.endpoint + "."
  private val incrFuncName = ns + "Incr" + config.funcName
  private val decrFuncName = ns + "Decr" + config.funcName
  private val setFuncName = ns + "Set" + config.funcName
  private val getFuncName = ns + "Get" + config.funcName

  val name: String = config.funcName

  def toCompile(typeContext: TypeContext): List[CompilerUnit] = {

    val keyTypes = config.keyTypes.asScala map { a => TypeCompiler.getType(a, typeContext) } toList

    val valueType = Type.LONG

    val valueSerializer = valueType.serializer

    def makeKey(args: util.List[AnyRef]) = {
      val hasher = Fingerprint.newHasher
      hasher.putUnencodedChars(classOf[MemCachedCounterOperator].getName)
      hasher.putUnencodedChars(config.dataset)
      keyTypes.zipWithIndex foreach {
        case (t, i) =>
          t.serializer.computeFingerprint(hasher, args.get(i))
      }
      val bytes = hasher.hash().asBytes()
      java.util.Base64.getUrlEncoder.encodeToString(bytes)
    }

    val incrFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes)),
        ImmutableList.of(Type.LONG),
        Type.LONG
      )

      @Override
      def funcName = incrFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"Increment the counter in Memcache. ${config.description}",
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[MemCachedEndpointContainer](incrFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[MemCachedEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val key = makeKey(args)
            val delta = if (args.size() > keyTypes.size) {
              args.get(keyTypes.size).asInstanceOf[java.lang.Long]
            } else {
              scala.Long.box(1L)
            }
            val endpoint = context.getRuntime.memCachedEndpoints(config.endpoint)
            endpoint.incr(key, delta) flatMap {
              case None =>
                val buf = Buf.Utf8(delta.toString)
                endpoint.set(key, buf).map(_ => delta)
              case Some(v) =>
                Future.value(scala.Long.box(v))
            }
          }
        }
      }
    }

    val decrFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes)),
        ImmutableList.of(Type.LONG),
        Type.LONG
      )

      @Override
      def funcName = decrFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"Decrement the counter Memcache. ${config.description}",
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[MemCachedEndpointContainer](decrFuncName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[MemCachedEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val key = makeKey(args)
            val delta = if (args.size() > keyTypes.size) {
              args.get(keyTypes.size).asInstanceOf[java.lang.Long]
            } else {
              scala.Long.box(1L)
            }

            val endpoint = context.getRuntime.memCachedEndpoints(config.endpoint)
            endpoint.decr(key, delta) map {
              case None => scala.Long.box(0L)
              case Some(v) => scala.Long.box(v)
            }

          }
        }
      }
    }

    val setFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes :+ valueType)),
        Type.UNIT
      )

      @Override
      def funcName = setFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"Set the counter value in Memcache. ${config.description}",
        ActionLevel.PROD_OTHER_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @Override
      @throws(classOf[SemanticCheckFailure])
      def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new GeneralNode[MemCachedEndpointContainer](setFuncName, children) {

          override protected def getCacheLevel = CacheLevel.Never

          override def getSignature = signature

          override protected def computeClassName = funcName

          override protected def evaluate(
            context: Context[MemCachedEndpointContainer],
            args: util.List[AnyRef]
          ): Future[AnyRef] = {
            val amount = args.get(keyTypes.size).asInstanceOf[java.lang.Long]
            if (amount < 0) {
              throw new IllegalArgumentException
            }
            val key = makeKey(args)
            val buf = Buf.Utf8(amount.toString)
            val endpoint = context.getRuntime.memCachedEndpoints(config.endpoint)
            endpoint.set(key, buf).voided
          }
        }
      }
    }

    val getFunc = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.copyOf(WrapAsJava.asJavaIterable(keyTypes)),
        valueType
      )

      @Override
      def funcName = getFuncName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"Get the counter value from Memcache. ${config.description}",
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
                java.lang.Long.valueOf(Buf.Utf8.unapply(buf).get)
              case None if args.size > keyTypes.size =>
                args.get(keyTypes.size)
              case None =>
                scala.Long.box(0L)
            }
          }
        }
      }
    }

    incrFunc :: decrFunc :: setFunc :: getFunc :: Nil
  }
}
