package com.twitter.botmaker.runtime

import java.io.File
import java.io.RandomAccessFile
import java.util
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.CompilerUnit
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode
import com.twitter.botmaker.runtime.config.LogWriterConfigs
import com.twitter.botmaker.runtime.thriftjava._
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.util._

case class LogWriterRegistry(config: LogWriterConfig) extends ServiceRegistry[LogWriterConfig] {

  override val name: String = config.funcName

  val dir: File = new File(config.rootDirectory, name)

  if (!dir.exists()) {
    dir.mkdirs
  }

  private var files = dir.listFiles().sortBy(f => f.lastModified).toList.reverse
  private var writer: Option[RandomAccessFile] = None

  def log(stats: Seq[AnyRef], timestamp: Long): Unit = {
    val w = ensure
    val str = "%s,(%s):%s\n".format(
      com.twitter.botmaker.runtime.config.LogWriter.timeFormatter.format(timestamp),
      stats.mkString(",")
    )
    w.writeBytes(str)
    w.getChannel.force(false)
  }

  private def ensure = {
    if (writer.isDefined && writer.get.length() >= config.maxLogFileSize) {
      Try {
        writer.get.close()
      }
      writer = None
    }

    writer.getOrElse {
      val fn = Try {
        if (files.length > 0) {
          files.head.getName.toInt
        } else {
          0
        }
      }.getOrElse(0)

      deleteOldFiles()

      val file = new File(dir, "%04d".format(fn + 1))
      files = file :: files

      writer = Some(new RandomAccessFile(file, "rw"))
      writer.get
    }
  }

  private def deleteOldFiles() = {
    files.drop(config.logFilesToKeep).foreach { file =>
      Try {
        file.delete
      }
    }

    files = files.take(config.logFilesToKeep)
  }

  override def close(deadline: Time): Future[Unit] = {
    Future.Unit.map { _ =>
      writer.map(_.close())
      writer = None
    }.unit
  }

  override def finalize(): Unit = {
    writer.map(_.close)
    super.finalize()
  }
}

trait LogWriterContainer extends ServiceContainer {
  container =>

  override type _CONFIGS_TYPE <: LogWriterConfigs

  private def isReusable(config: LogWriterConfig) =
    isReusable[LogWriterConfig, LogWriterRegistry](config.funcName, config)

  def logWriters(config: LogWriterConfig): LogWriterRegistry = {
    serviceRegistry[LogWriterRegistry](config.funcName).get
  }

  override def build(configs: _CONFIGS_TYPE, scoped: StatsReceiver): Seq[ServiceRegistry[Any]] = {
    val logWriters = configs.logWriters.map {
      case config if isReusable(config) =>
        logger.info(s"reuse log writers ${config.funcName}")
        serviceRegistry[LogWriterRegistry](config.funcName).get
      case config =>
        logger.info(s"create log writer ${config.funcName}")
        LogWriterRegistry(config)
    }

    logWriters ++ super.build(configs, scoped)
  }
}

case class LogWriterOperator(config: LogWriterConfig) {

  val name: String = config.funcName

  def toCompile: List[CompilerUnit] = {
    val func = new CompilerUnit {

      private val signature = new Signature(
        ImmutableList.of[Type],
        Type.OBJECT,
        Type.UNIT
      )

      override def funcName = config.funcName

      override def botMakerDoc: BotMakerDoc = BotMakerDoc.of(
        funcName,
        signature,
        s"Write log to local files ${config.funcName}. ${config.description}",
        ActionLevel.NO_ACTION
      )

      override def fingerprint(): Fingerprint = Fingerprint.of(config)

      @throws(classOf[SemanticCheckFailure])
      override def createASTNode(children: ImmutableList[ASTNode[_]]) = {

        new FunctionNode[LogWriterContainer](funcName, children) {

          override def getSignature = signature

          override protected def getCacheLevel = CacheLevel.Never

          override protected def computeClassName = funcName

          override protected def apply(
            context: Context[LogWriterContainer],
            args: util.List[AnyRef]
          ) = {
            val writer = context.getRuntime.logWriters(config)
            writer.log(args.asScala, context.getRuntime.clock.nowMillis)
            unit
          }
        }
      }
    }

    List(func)
  }
}
