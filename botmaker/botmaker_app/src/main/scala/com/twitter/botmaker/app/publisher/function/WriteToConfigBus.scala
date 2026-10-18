package com.twitter.botmaker.app.publisher.function

import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.app.publisher.BotMakerPublisherRuntime
import com.twitter.botmaker.common.SerializerUtil
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.GeneralNode
import com.twitter.botmaker.rule.thriftjava.BotMakerRulesPackage
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.configbus.common.model.EntryType
import com.twitter.configbus.common.model.Op
import com.twitter.configbus.common.model.Update
import com.twitter.io.Buf
import com.twitter.util.Future

import java.util

@BotMakerFunction(
  argTypes = Array("String", "BotMakerRulesPackage", "UserName"),
  arguments = Array("app name", "rules package", "[user name]"),
  returnType = "String",
  actionLevel = ActionLevel.PROD_OTHER_ACTION,
  description = "Write botmaker rules package of the given app to config bus under " +
    "botmaker/[username|default]/<environment>/<app>/<app>.json and returns the gitCommitSha."
)
class WriteToConfigBus(exprText: String, children: ImmutableList[ASTNode[_]])
    extends GeneralNode[BotMakerPublisherRuntime](
      exprText,
      children
    ) {

  override protected def getCacheLevel = CacheLevel.Never

  override def getSignature: Signature = new Signature(
    ImmutableList.of(Type.STRING, Type.thriftOf(classOf[BotMakerRulesPackage])),
    ImmutableList.of(Type.STRING),
    Type.STRING
  )

  override protected def evaluate(
    context: Context[BotMakerPublisherRuntime],
    args: util.List[AnyRef]
  ): Future[AnyRef] = {

    val appName = args.get(0).asInstanceOf[String]
    val rulesPackage = args.get(1).asInstanceOf[BotMakerRulesPackage]
    val userName = if (args.size() >= 3) args.get(2).asInstanceOf[String] else "default"

    val runtime = context.getRuntime
    val text = SerializerUtil.toJson(rulesPackage)

    val file = "botmaker/%s/%s/%s/%s.json".format(
      userName,
      runtime.info.get.environment,
      appName,
      appName
    )

    val update = Update(
      Op.Createorupdate,
      EntryType.File,
      file,
      Some(Buf.Utf8(text)),
      None
    )

    runtime.configbusClient
      .compareAndSet(Seq(update), s"Updating rule package")
      .map { result => "sha:%s, opResult:%s".format(result.gitCommitSha, result.opResult.name) }
  }
}
