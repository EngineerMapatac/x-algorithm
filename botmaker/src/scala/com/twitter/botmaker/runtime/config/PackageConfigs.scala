package com.twitter.botmaker.runtime.config

import scala.collection.mutable
import scala.reflect.ClassTag
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerFunction
import com.twitter.botmaker.compiler.CompilerBuilder
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.thrift.ThriftOperator
import com.twitter.botmaker.runtime.thriftjava.{PackageConfig => PackageConfigDef}
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.compiler.types.ThriftType

object PackageConfig {
  val TPA: Type = Type.newGenericType
  val TPB: Type = Type.newGenericType
  val thriftType: ThriftType = Type.thriftOf(classOf[PackageConfigDef])
  val signature: ASTNode.Signature = new ASTNode.Signature(Type.pairOf(TPA, TPB), thriftType)
}

@BotMakerFunction(
  argTypes = Array("Pair<A, B> ..."),
  arguments = Array("key / value pairs"),
  returnType = "botmaker function package config value",
  description = "Add a botmaker function package config.",
  actionLevel = ActionLevel.NO_ACTION
)
class PackageConfig[E <: PackageConfigs](exprText: String, children: ImmutableList[ASTNode[_]])
    extends ThriftOperator[E, PackageConfigDef](
      classOf[PackageConfigDef],
      exprText,
      children
    ) {

  override def getSignature: Signature = PackageConfig.signature

  override protected def apply(context: Context[E], config: PackageConfigDef): AnyRef = {
    context.getRuntime.packageConfigs += config
    config
  }
}

trait PackageConfigs extends ServiceConfigs {

  val packageConfigs: mutable.ArrayBuffer[PackageConfigDef] =
    mutable.ArrayBuffer[PackageConfigDef]()

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]
    packageConfigs foreach { p => builder.addPackage(p.packageName) }
    builder
  }
}
