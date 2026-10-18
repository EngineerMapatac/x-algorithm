package com.twitter.botmaker.runtime.config

import com.google.common.base.Optional
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.StructTupleType
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.compiler.types.TypeCompiler
import com.twitter.botmaker.compiler.types.TypeContext
import com.twitter.botmaker.config.ConfigContainer
import com.twitter.botmaker.function.FunctionNode0
import com.twitter.botmaker.runtime.TwitterRuntime.Thrift
import com.twitter.botmaker.runtime._
import com.twitter.botmaker.runtime.thriftjava.StructTupleFieldConfig
import com.twitter.botmaker.runtime.thriftjava.StructTupleTypeConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftTypeConfig
import com.twitter.scrooge.ThriftStruct
import scala.collection.JavaConverters._
import scala.collection.mutable
import scala.reflect.ClassTag
import scala.util.matching.Regex

object ConfigValue
object FuncName
object Endpoint
object TypeName

abstract class ServiceConfigs extends ConfigContainer with Debuggable {

  private final val container = mutable.Map[String, (Type, String, AnyRef)]()
  private final val uniqueNames = mutable.Set[Seq[Any]]()
  private final val thriftTypeConfigs = mutable.Map[String, ThriftTypeConfig]()
  private final val structTupleTypeConfigs = mutable.Map[String, StructTupleTypeConfig]()

  val eventTypeRegex: Regex = "^[a-z][a-z0-9_]*$".r
  val fieldNameRegex: Regex = "^[a-zA-Z_][a-zA-Z0-9_]*$".r
  val funcNameRegex: Regex = "^[A-Z][a-zA-Z0-9_]*$".r
  val endpointRegex: Regex = "^[a-z][a-zA-Z0-9_]*$".r
  val datasetRegex: Regex = "^[a-zA-Z_][a-zA-Z0-9_]*$".r
  val typeNameRegex: Regex = "^[A-Z][a-zA-Z0-9_]*$".r
  val teamNameRegex: Regex = "^[a-z][a-z0-9\\-]*$".r

  def unique(names: Any*): Unit = synchronized {
    val nameSeq = names.toSeq
    if (uniqueNames.contains(nameSeq)) {
      throw ConfigFailure(
        s"Conflict config name:${nameSeq.mkString}"
      )
    }
    uniqueNames += nameSeq
  }

  def validate(condition: Boolean, err: String): Unit = {
    if (!condition) {
      throw ConfigFailure(err)
    }
  }

  def validateEventType(eventType: String): Unit = {
    if (eventTypeRegex.findFirstIn(eventType).isEmpty) {
      throw ConfigFailure(
        s"Event type '$eventType' does not match $eventTypeRegex."
      )
    }
  }

  def validateFieldName(fieldName: String): Unit = {
    if (fieldNameRegex.findFirstIn(fieldName).isEmpty) {
      throw ConfigFailure(
        s"Field name '$fieldName' does not match $fieldNameRegex."
      )
    }
  }

  def validateFuncName(funcName: String): Unit = {
    if (funcNameRegex.findFirstIn(funcName).isEmpty) {
      throw ConfigFailure(
        s"Func name '$funcName' does not match $funcNameRegex."
      )
    }
  }

  def validateEndpoint(endpoint: String): Unit = {
    if (endpoint != "ScribeClient"
      && endpointRegex.findFirstIn(endpoint).isEmpty) {
      throw ConfigFailure(
        s"Endpoint '$endpoint' does not match $endpointRegex."
      )
    }
  }

  def validateDataset(dataset: String): Unit = {
    if (datasetRegex.findFirstIn(dataset).isEmpty) {
      throw ConfigFailure(
        s"Dataset '$dataset' does not match $datasetRegex."
      )
    }
  }

  def validateTypeName(typename: String): Unit = {
    if (typeNameRegex.findFirstIn(typename).isEmpty) {
      throw ConfigFailure(
        s"TypeName '$typename' does not match $typeNameRegex."
      )
    }
  }

  def validateTeamName(teamName: String): Unit = {
    if (teamNameRegex.findFirstIn(teamName).isEmpty) {
      throw ConfigFailure(
        s"Team name '$teamName' does not match $teamNameRegex."
      )
    }
  }

  def validateClassName(className: String): Unit = {
    ClassCache.forName(className)
  }

  override def addConfig(
    configName: String,
    configType: Type,
    description: String,
    configValue: AnyRef
  ): Unit = synchronized {
    unique(configValue, configName)
    container.put(configName, (configType, description, configValue))
  }

  final def addThriftType(config: ThriftTypeConfig): Unit = synchronized {
    unique(TypeName, config.typeName)
    validateTypeName(config.typeName)
    thriftTypeConfigs.put(config.typeName, config)
  }

  final def addStructTupleType(config: StructTupleTypeConfig): Unit = synchronized {
    unique(TypeName, config.typeName)
    validateTypeName(config.typeName)
    config.fields.asScala foreach { f => validateFieldName(f.fieldName) }

    structTupleTypeConfigs.put(config.typeName, config)
  }

  def validate: Seq[Throwable] = {
    val errors = thriftTypeConfigs.toSeq collect {
      case (typeName: String, thriftTypeConfig: ThriftTypeConfig)
          if !isThriftType(thriftTypeConfig.className) =>
        ConfigFailure(
          s"invalid class name ${thriftTypeConfig.className} in thrift type ${typeName}"
        )
    }
    errors
  }

  private def isThriftType(className: String): Boolean = {
    classOf[Thrift[_, _]].isAssignableFrom(ClassCache.forName(className)) ||
    classOf[ThriftStruct].isAssignableFrom(ClassCache.forName(className))
  }

  def debug(dbg: RuntimeDebugger): Unit = {
    container foreach {
      case (name, tuple) =>
        dbg.info(name, tuple.toString())
    }

    dbg.open("thriftTypes", thriftTypeConfigs) foreach { debugger: RuntimeDebugger =>
      thriftTypeConfigs foreach {
        case (name: String, config: ThriftTypeConfig) =>
          debugger.info(name, config.toString)
      }
    }

    dbg.open("structTupleTypes", structTupleTypeConfigs) foreach { debugger: RuntimeDebugger =>
      structTupleTypeConfigs foreach {
        case (name: String, config: StructTupleTypeConfig) =>
          debugger.info(name, config.toString)
      }
    }
  }

  def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val validateErrors = validate
    if (validateErrors.nonEmpty) {
      val msg = validateErrors.take(5) map {
        _.toString
      } mkString "\n"
      throw new SemanticCheckFailure(s"Failed to validate service config: $msg")
    }

    val classTag = implicitly[ClassTag[E]]
    val runtimeClass = classTag.runtimeClass.asInstanceOf[Class[_ <: E]]
    val builder = new CompilerBuilder[E](runtimeClass)
    builder.addPackage("com.twitter.botmaker.function")
    builder.addPackage("com.twitter.botmaker.runtime.function")

    addTypesFromConfigs(builder)
    addConfigValueFunctions(builder)

    builder
  }

  private def addTypesFromConfigs[E](builder: CompilerBuilder[E]): Unit = {

    val typeContext = new TypeContext {

      val parsedAndParentTypeNames = mutable.Set[String]()
      val parsedNamedTypes = mutable.Map[String, Type]()

      override def getTypeFromName(typeName: String): Optional[Type] = {
        if (parsedNamedTypes.contains(typeName)) {
          Optional.of(parsedNamedTypes(typeName))
        } else if (parsedAndParentTypeNames.contains(typeName)) {
          Optional.absent()
        } else {
          parsedAndParentTypeNames += typeName
          if (thriftTypeConfigs.contains(typeName)) {
            val tc: ThriftTypeConfig = thriftTypeConfigs(typeName)
            val clazz = ClassCache.forName(tc.className)
            if (classOf[Thrift[_, _]].isAssignableFrom(clazz)) {
              val thriftClass = clazz.asInstanceOf[Class[_ <: Thrift[_, _]]]
              val theType = Type.thriftOf(thriftClass)
              parsedNamedTypes.put(typeName, theType)
              Optional.of(theType)
            } else if (classOf[ThriftStruct].isAssignableFrom(clazz)) {
              val thriftStructClass = clazz.asInstanceOf[Class[_ <: ThriftStruct]]
              val theType = Type.thriftStructOf(thriftStructClass)
              parsedNamedTypes.put(typeName, theType)
              Optional.of(theType)
            } else {
              Optional.absent()
            }
          } else if (structTupleTypeConfigs.contains(typeName)) {
            val sc: StructTupleTypeConfig = structTupleTypeConfigs(typeName)
            val fields: Seq[(String, Type)] = sc.fields.asScala map { f: StructTupleFieldConfig =>
              f.fieldName -> TypeCompiler.getType(f.fieldType, this)
            }
            val theType = Type.structTupleOf(
              ImmutableList.copyOf(fields.map(_._1).toArray),
              ImmutableList.copyOf(fields.map(_._2).toArray)
            )
            parsedNamedTypes.put(typeName, theType)
            Optional.of(theType)
          } else {
            Optional.absent()
          }
        }
      }
    }

    thriftTypeConfigs.values foreach { tc: ThriftTypeConfig =>
      val clazz = ClassCache.forName(tc.className)
      if (classOf[Thrift[_, _]].isAssignableFrom(clazz)) {
        val thriftClass = clazz.asInstanceOf[Class[_ <: Thrift[_, _]]]
        builder.addThriftType(thriftClass, tc.typeName)
      } else if (classOf[ThriftStruct].isAssignableFrom(clazz)) {
        val thriftClass = clazz.asInstanceOf[Class[_ <: ThriftStruct]]
        builder.addThriftStructType(thriftClass, tc.typeName)
      }
    }

    structTupleTypeConfigs.values foreach { sc: StructTupleTypeConfig =>
      val typeOpt: Optional[Type] = typeContext.getTypeFromName(sc.typeName)
      if (!typeOpt.isPresent) {
        throw new ConfigFailure(s"Invalid StructTupleType config ${sc.typeName}")
      }
      builder.addStructTupleType(typeOpt.get.asInstanceOf[StructTupleType], sc.typeName)
    }
  }

  private def addConfigValueFunctions[E](builder: CompilerBuilder[E]): Unit = {
    container foreach {
      case (configName, (configType, description, configValue)) =>
        val signature = new ASTNode.Signature(
          ImmutableList.of[Type](),
          configType
        )
        val ns = "cfg"
        val cu = new CompilerUnit {
          override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] =
            new FunctionNode0[Runtime](configName, children) {

              override protected def apply(context: Context[Runtime]) = configValue

              override def getSignature = signature

              override protected def getCacheLevel = ASTNode.CacheLevel.Global
            }

          override def botMakerDoc(): BotMakerDoc = BotMakerDoc.of(
            funcName,
            signature,
            description,
            ActionLevel.NO_ACTION
          )

          override def funcName(): String = s"$ns.$configName"

          override def fingerprint(): Fingerprint = {
            val hasher = Fingerprint.newHasher
            hasher.putUnencodedChars(classOf[ServiceConfigs].getName)
            hasher.putUnencodedChars(configName)
            new Fingerprint(0, hasher.hash.asBytes)
          }
        }

        builder.addCompilerUnit(cu)
    }
  }
}
