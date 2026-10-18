package com.twitter.botmaker.app.publisher.function

import java.io.File
import java.io.FileWriter
import java.nio.file.Files
import java.nio.file.Paths
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.app.publisher.BotMakerPublisherRuntime
import com.twitter.botmaker.common.SerializerUtil
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.FunctionNode2
import com.twitter.botmaker.rule.thriftjava.BotMakerRulesPackage
import com.twitter.botmaker.runtime.RuntimeDebugger
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.util.Try

@BotMakerFunction(
  argTypes = Array("String", "BotMakerRulesPackage"),
  arguments = Array("file path", "rules package"),
  returnType = "Unit",
  description = "Write botmaker rules package to a local file",
  actionLevel = ActionLevel.NO_ACTION
)
class WriteToFile(exprText: String, children: ImmutableList[ASTNode[_]])
    extends FunctionNode2[BotMakerPublisherRuntime, String, BotMakerRulesPackage](
      exprText,
      children
    ) {

  override protected def getCacheLevel = CacheLevel.Never

  override def getSignature: Signature = new Signature(
    ImmutableList.of(Type.STRING, Type.thriftOf(classOf[BotMakerRulesPackage])),
    Type.UNIT
  )

  override protected def apply(
    context: Context[BotMakerPublisherRuntime],
    file: String,
    rulesPackage: BotMakerRulesPackage
  ): AnyRef = {

    val runtime = context.getRuntime
    val text = SerializerUtil.toJson(rulesPackage)

    val infos = RuntimeDebugger.infos(runtime, "runtime", "info", "loader")
    val baseConfigPath = infos("runtime.info.loader.baseConfigPath")
    val temp = Paths.get(baseConfigPath, file + ".tmp").toAbsolutePath.toString
    val target = Paths.get(baseConfigPath, file).toAbsolutePath.toString

    try {

      val src = new File(temp)
      val dst = new File(target)
      src.getParentFile.mkdirs

      val writer = new FileWriter(temp)
      writer.write(text)
      writer.close()

      if (!src.renameTo(dst)) {
        var srcPath = Paths.get(temp)
        var dstPath = Paths.get(target)
        Files.copy(srcPath, dstPath)
      }

      runtime.logger.info(s"publish to $target")

      unit
    } finally {
      Try {
        Files.deleteIfExists(Paths.get(temp))
      }
    }
  }
}
