package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import scala.reflect.ClassTag
import com.google.common.base.Strings
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.thrift.ThriftOperator
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.BatchQueueConfig
import com.twitter.botmaker.runtime.thriftjava.BotMakerEventTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftFieldConfig
import com.twitter.botmaker.compiler.types.ThriftType

object BatchQueue {
  val TPA: Type = Type.newGenericType
  val TPB: Type = Type.newGenericType
  val thriftType: ThriftType = Type.thriftOf(classOf[BatchQueueConfig])
  val signature: ASTNode.Signature = new ASTNode.Signature(Type.pairOf(TPA, TPB), thriftType)
}

@BotMakerFunction(
  argTypes = Array("Pair<A, B> ..."),
  arguments = Array("key / value pairs"),
  returnType = "Batch Queue config value",
  description = "Batch Queue config.",
  actionLevel = ActionLevel.NO_ACTION
)
class BatchQueue[E <: BatchQueueConfigs](exprText: String, children: ImmutableList[ASTNode[_]])
    extends ThriftOperator[E, BatchQueueConfig](
      classOf[BatchQueueConfig],
      exprText,
      children
    ) {

  override protected def getCacheLevel = CacheLevel.Never
  override def getSignature: Signature = BatchQueue.signature
  override protected def apply(context: Context[E], config: BatchQueueConfig): AnyRef = {

    context.getRuntime.addBatchQueue(config)
    config
  }
}

trait BatchQueueConfigs extends ServiceConfigs with EventTypeConfigs {

  private final val batchQueues = mutable.ArrayBuffer[BatchQueueConfig]()

  def getBatchQueues: Seq[BatchQueueConfig] = batchQueues.toSeq

  def addBatchQueue(config: BatchQueueConfig): Option[BotMakerEventTypeConfig] = {

    unique(Endpoint, config.eventType)
    unique(BatchQueue, config.eventType)

    validateEventType(config.eventType)

    validate(
      config.batchSize > 0,
      s"invalid batchSize ${config.batchSize} in batchqueue ${config.eventType}"
    )

    validate(
      config.concurrentSize > 0,
      s"invalid concurrentSize ${config.concurrentSize} in batchqueue ${config.eventType}"
    )

    validate(
      config.queueSize > 0,
      s"invalid queueSize ${config.queueSize} in batchqueue ${config.eventType}"
    )

    batchQueues.append(config)

    addEventType(
      new BotMakerEventTypeConfig()
        .setEventType(config.eventType)
        .setInputFeatures(
          ImmutableList.of(
            new ThriftFieldConfig()
              .setFieldName("batch")
              .setFieldId(1)
              .setFieldType(s"List<${config.itemType}>")
          )
        )
    )
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    val tc = builder.typeContext()

    batchQueues foreach {
      case bc if Strings.isNullOrEmpty(bc.returnType) =>
        val p = com.twitter.botmaker.runtime.OneWayBatchQueueOperator(bc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
      case bc if bc.returnType == "Unit" =>
        val p = com.twitter.botmaker.runtime.UnitBatchQueueOperator(bc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
      case bc =>
        val p = com.twitter.botmaker.runtime.BatchQueueOperator(bc)
        p.toCompile(tc).foreach(u => builder.addCompilerUnit(u))
    }

    builder
  }
}
