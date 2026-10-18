package com.twitter.botmaker.runtime

import java.util
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.botmaker.runtime.config.EventPublisherConfigs
import com.twitter.botmaker.runtime.thriftjava.EventPublisherConfig
import com.twitter.eventbus.client.EventBusPublisher
import com.twitter.eventbus.client.EventBusPublisherBuilder
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util.Future
import com.twitter.util.Time

case class EventPublisherRegistry(
  config: EventPublisherConfig,
  publisher: EventBusPublisher[Thrift[_, _]])
    extends ServiceRegistry[EventPublisherConfig] {

  override val name: String = config.funcName

  override def close(deadline: Time): Future[Unit] = publisher.close(deadline)
}

trait EventBusPublisherFactory extends TwitterRuntime {

  def build(
    config: EventPublisherConfig,
    partitioner: Option[(Thrift[_, _], Int) => Int],
    scoped: StatsReceiver
  ): EventBusPublisher[Thrift[_, _]] = {

    var builder = EventBusPublisherBuilder()
      .clientId(config.clientId)
      .streamName(config.streamName)
      .statsReceiver(scoped.scope(config.funcName))
      .maxQueuedEvents(config.maxQueuedEvents)

    if (config.isSetKafkaDest) {
      builder = builder.kafkaDest(config.kafkaDest)
    }

    if (partitioner.isDefined) {
      builder
        .thriftStruct()
        .partitioner(partitioner.get)
        .build()
    } else {
      builder
        .thriftStruct()
        .build()
    }
  }
}

trait EventPublisherContainer extends ServiceContainer with EventBusPublisherFactory {
  container =>

  override type _CONFIGS_TYPE <: EventPublisherConfigs

  private def isReusable(config: EventPublisherConfig) =
    isReusable[EventPublisherConfig, EventPublisherRegistry](config.funcName, config)

  def eventPublishers(config: EventPublisherConfig): EventBusPublisher[Thrift[_, _]] = {
    serviceRegistry[EventPublisherRegistry](config.funcName).get.publisher
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val eventPublishers = configs.getEventPublishers map {
      case config if isReusable(config) =>
        logger.info(s"reuse EventBusPublisher ${config.funcName}")
        serviceRegistry[EventPublisherRegistry](config.funcName).get
      case config =>
        logger.info(s"create EventBusPublisher ${config.funcName}")

        val partitioner = if (config.partitionFields.size > 0) {
          val thriftClass = ClassCache.forName(config.thriftClass).asInstanceOf[Class[Thrift[_, _]]]
          val thriftType = Type.thriftOf(thriftClass)
          val fieldTypes = config.partitionFields.asScala map { fieldName =>
            fieldName -> thriftType.getFieldType(fieldName)
          }

          val fn = (thrift: Thrift[_, _], pn: Int) => {
            val hasher = Fingerprint.newHasher
            fieldTypes foreach {
              case (fieldName, fieldType) =>
                val fieldValue = thriftType.getFieldValue(thrift, fieldName)
                if (fieldValue == null) {
                  hasher.putLong(0L)
                } else {
                  fieldType.serializer.computeFingerprint(hasher, fieldValue)
                }
            }

            Math.abs(hasher.hash.asInt) % pn
          }
          Some(fn)
        } else {
          None
        }
        val publisher = build(config, partitioner, scoped.scope("EventPublisher"))
        EventPublisherRegistry(config, publisher)
    }
    eventPublishers ++ super.build(configs, scoped)
  }
}

case class EventPublisherOperator(config: EventPublisherConfig) {

  private val thriftClass = ClassCache.forName(config.thriftClass).asInstanceOf[Class[Thrift[_, _]]]
  private val thriftType = Type.thriftOf(thriftClass)

  config.partitionFields.asScala map { fieldName =>
    fieldName -> thriftType.getFieldType(fieldName)
  }

  private val ns = "eb."
  private val funcName: String = ns + config.funcName

  def toCompile: CompilerUnit = new CompilerUnit {

    private val signature = new Signature(
      ImmutableList.of[Type](thriftType),
      Type.UNIT
    )

    override def funcName: String = EventPublisherOperator.this.funcName

    override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
      funcName,
      signature,
      s"publish an event thrift object to eventbus: {$config.description}",
      ActionLevel.PROD_OTHER_ACTION
    )

    override def fingerprint(): Fingerprint = Fingerprint.of(config)

    @throws(classOf[SemanticCheckFailure])
    override def createASTNode(
      children: ImmutableList[ASTNode[_]]
    ): GeneralNode[EventPublisherContainer] = {

      new GeneralNode[EventPublisherContainer](funcName, children) {

        override def getSignature: Signature = signature

        override protected def getCacheLevel = CacheLevel.Never

        override protected def computeClassName: String = funcName

        override protected def evaluate(
          context: Context[EventPublisherContainer],
          args: util.List[AnyRef]
        ): Future[AnyRef] = {
          val publisher = context.getRuntime.eventPublishers(config)
          val thriftObject = args.get(0).asInstanceOf[Thrift[_, _]]
          publisher.publish(thriftObject).voided
        }
      }
    }
  }
}
