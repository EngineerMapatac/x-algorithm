package com.twitter.botmaker

import java.util.{List => JList}
import scala.collection.JavaConverters._
import org.apache.thrift.TBase
import org.apache.thrift.TFieldIdEnum
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.BotMakerDoc
import com.twitter.botmaker.compiler.CompilerUnit
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.exceptions.SemanticCheckFailure
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.thrift.ThriftOperator
import com.twitter.botmaker.function._
import com.twitter.botmaker.rule.RuleContext
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.compiler.types.ThriftType
import com.twitter.util.Future

trait DownstreamService {
  val name: String
  override def toString: String = name
}

trait FunctionUnit extends CompilerUnit {

  lazy private[this] val funcName0 = getClass.getSimpleName.stripSuffix("$")
  lazy private[this] val packageName0 = getClass.getPackage.getName

  override def funcName: String = funcName0

  def packageName: String = packageName0

  override def enabled: Boolean = true

  def cacheLevel: ASTNode.CacheLevel

  def actionLevel: ActionLevel

  def downstreams: Set[DownstreamService]

  def description: String

  def arguments: Seq[String]

  def examples: Seq[String] = Nil

  def shouldApplyRateLimit: Boolean =
    this.actionLevel == ActionLevel.PROD_TANGIBLE_IMPACT_ACTION ||
      this.actionLevel == ActionLevel.PROD_AGENT_WORKFLOW_ACTION
}

object FunctionUnit {
  val TPA: Type = Type.newGenericType
  val TPB: Type = Type.newGenericType

  trait SignatureChecker extends CompilerUnit { self: FunctionUnit =>

    def signature: Signature

    def validateSignature(children: Seq[ASTNode[_]]): Unit = {}

    def assertConstant(node: ASTNode[_]): Unit = {
      node match {
        case constant: Constant =>
          ()
        case _ =>
          throw new SemanticCheckFailure(
            s"type checking expression ${node.getExprText} failed: invalid argument type: expected a constant ${node.getReturnType}"
          )
      }
    }

    abstract override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
      validateSignature(children.asScala)
      super.createASTNode(children)
    }
  }

  def byNameToByValue[T](func: => T): () => T = () => func

  trait Managed extends FunctionUnit { self: FunctionUnit =>
    def manage[E, T](context: Context[E], args: JList[AnyRef])(func: => T): T = {
      context.reportFunctionEvaluation(funcName, actionLevel, args)
      func
    }

    def manageFuture[E](
      context: Context[E],
      args: JList[AnyRef]
    )(
      func: => Future[AnyRef]
    ): Future[AnyRef] = {
      context.reportFunctionEvaluation(funcName, actionLevel, args)
      if (this.shouldApplyRateLimit) {
        context.applyActionRateLimit(
          context,
          funcName,
          actionLevel,
          byNameToByValue { func },
          ImmutableList.of())
      } else {
        func
      }
    }
  }

  trait Cached extends Managed { self: FunctionUnit =>

    override def manage[E, T](context: Context[E], args: JList[AnyRef])(func: => T): T = {
      context.getRootContext match {
        case r: RuleContext[_] =>
          r.eventContext.cacheGet((self, args)) {
            super.manage(context, args)(func)
          }
        case _ =>
          super.manage(context, args)(func)
      }
    }

    override def manageFuture[E](
      context: Context[E],
      args: JList[AnyRef]
    )(
      func: => Future[AnyRef]
    ): Future[AnyRef] = {
      context.getRootContext match {
        case r: RuleContext[_] =>
          r.eventContext.cacheGet((self, args)) {
            super.manageFuture(context, args)(func)
          }
        case _ =>
          super.manageFuture(context, args)(func)
      }
    }
  }

  def codeGen(paramSize: Int, optParamSize: Int, varArgs: Boolean = false): List[String] = {

    if (varArgs) {
      require(optParamSize == 0, "Optional parameters not compatible with varArgs")
    }

    val totalParamSize = paramSize + optParamSize
    val ts = (1 to totalParamSize) map { i => s"T$i" }

    val ifVarArgs = (s: String) =>
      if (varArgs) { s }
      else { "" }
    val commaSeperate = (l: List[String]) => l.filterNot(_.isEmpty).mkString(",")

    val someArgs = (args: List[String]) => {
      ((1 to paramSize) map { i => args.asJava.get(i - 1) }).toList :::
        ((paramSize + 1 to args.length) map { i => s"Some(${args.asJava.get(i - 1)})" }).toList
    }

    val funcCode = if (optParamSize == 0 && varArgs) {
      s"${paramSize}V"
    } else if (optParamSize == 0) {
      s"$paramSize"
    } else {
      s"${paramSize}O$optParamSize"
    }

    val funcName = s"FunctionUnit$funcCode"

    val optParamComment = if (varArgs) {
      s"variable"
    } else {
      s"$optParamSize optional"
    }

    val generics = List("E") ::: ts.toList ::: List(ifVarArgs("V"), "R")

    val args: List[String] = ts.map { t => s"${t.toLowerCase}: Manifest[$t]," }.toList ::: List(
      ifVarArgs("v: Manifest[V],"),
      "r: Manifest[R]"
    ).filterNot(_.isEmpty)

    val resultType = "R"

    val evaluateSigArgs: List[String] =
      ((1 to paramSize) map { i: Int => s"arg$i: T$i" }).toList :::
        ((1 to optParamSize) map { i: Int =>
          s"opt$i: Option[T${paramSize + i}] = None"
        }).toList ::: List(
        ifVarArgs("vargs: Seq[V]")
      )

    val bodyArgs = (n: Int) =>
      commaSeperate(
        List("context") :::
          someArgs(((1 to n) map { i =>
            s"toScala$i(args.get(${i - 1})).asInstanceOf[T$i]"
          }).toList) :::
          List(
            ifVarArgs(s"args.asScala.drop($paramSize).map(toScalaV(_).asInstanceOf[V])")
          )
      )

    val evaluateSignature: String =
      s"def evaluate(${commaSeperate(List("context: Context[E]") ::: evaluateSigArgs)}): $resultType"

    val astNode = {
      val evalFunc = s"$funcName.this.evaluate"
      def body(addFuture: Boolean = false): List[String] = if (optParamSize == 0) {
        List(
          if (addFuture) s"val result = Future($evalFunc(${bodyArgs(paramSize)}))"
          else s"val result = $evalFunc(${bodyArgs(paramSize)})"
        )
      } else {
        ((paramSize to paramSize + optParamSize).toList flatMap {
          case pi if pi == paramSize =>
            List(
              s"val result = if (args.size == $pi) {",
              if (addFuture) s"Future($evalFunc(${bodyArgs(pi)}))"
              else s"$evalFunc(${bodyArgs(pi)})"
            )
          case pi if pi < paramSize + optParamSize =>
            List(
              s"} else if (args.size == $pi) {",
              if (addFuture) s"Future($evalFunc(${bodyArgs(pi)}))"
              else s"$evalFunc(${bodyArgs(pi)})"
            )
          case pi =>
            List(
              s"} else {",
              if (addFuture) s"Future($evalFunc(${bodyArgs(pi)}))"
              else s"$evalFunc(${bodyArgs(pi)})"
            )
        }) ::: List(
          "}"
        )
      }

      List(
        "if (isAsync) {"
      ) ::: List(
        "new GeneralNode[E](funcName(), children) {",
        "override def getSignature: Signature = signature",
        "override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel",
        "override protected def computeClassName: String = funcName",
        "override protected def computePackageName: String = packageName",
        "",
        "override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] = manageFuture(context, args) {"
      ) ::: body() ::: List(
        "result.asInstanceOf[Future[AnyRef]] map { toJava(_) }",
        "}",
        "}",
        s"} else if ($funcName.this.shouldApplyRateLimit) {"
      ) ::: List(
        "new GeneralNode[E](funcName(), children) {",
        "override def getSignature: Signature = signature",
        "override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel",
        "override protected def computeClassName: String = funcName",
        "override protected def computePackageName: String = packageName",
        "",
        "override protected def evaluate(context: Context[E], args: JList[AnyRef]): Future[AnyRef] = manageFuture(context, args) {"
      ) ::: body(addFuture = true) ::: List(
        "result.map(toJava)",
        "}",
        "}",
        "} else {"
      ) ::: List(
        "new FunctionNode[E](funcName(), children) {",
        "override def getSignature: Signature = signature",
        "override protected def getCacheLevel: ASTNode.CacheLevel = cacheLevel",
        "override protected def computeClassName: String = funcName",
        "override protected def computePackageName: String = packageName",
        "",
        "override protected def apply(context: Context[E], args: JList[AnyRef]): AnyRef = manage(context, args) {"
      ) ::: body() ::: List(
        "toJava(result)",
        "}",
        "}"
      )
    }

    List(
      s"/**",
      s"* $funcName with $paramSize required arguments and $optParamComment arguments",
      s"*/",
      s"abstract class $funcName[${commaSeperate(generics)}](",
      s"implicit " + args.head
    ) ::: args.drop(1) ::: List(
      ") extends FunctionUnit.Managed {"
    ) ::: List(
      "",
      "private[this] val isAsync = classOf[Future[_]].isAssignableFrom(r.runtimeClass)",
      "private[this] val r0 = if (isAsync) r.typeArguments.head else r",
      "val signature: Signature = new Signature(",
      s"ImmutableList.of[Type](${(1 to paramSize).toList.map(i => s"ofType[T$i]").mkString(",")}),",
      if (varArgs) { "ofType[V]," }
      else {
        s"ImmutableList.of[Type](${(paramSize + 1 to paramSize + optParamSize).toList
          .map(i => s"ofType[T$i]")
          .mkString(",")}),"
      },
      "ofType(r0)",
      ")",
      "override val botMakerDoc: BotMakerDoc = BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toList: _*)",
      "override def fingerprint(): Fingerprint = Fingerprint.EMPTY",
      ""
    ) ::: (ts.zipWithIndex map {
      case (_, i) =>
        s"private[this] val toScala${i + 1}: AnyRef => Any = asScala[T${i + 1}]"
    }).toList ::: {
      if (varArgs) { List("val toScalaV: AnyRef => Any = asScala[V]") }
      else { Nil }
    } ::: List(
      "private[this] val toJava: Any => AnyRef = asJava(r0)",
      "",
      evaluateSignature,
      ""
    ) ::: List(
      "override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {"
    ) ::: astNode ::: List(
      "}",
      "}",
      "}"
    )
  }

  def codeGenPreset(): List[String] = {
    List(
      """
package com.twitter.botmaker

import java.util.{List => JList}
import scala.collection.JavaConverters._
import com.google.common.collect.ImmutableList
import com.twitter.botmaker.ASTNode.Signature
import com.twitter.botmaker.compiler.{BotMakerDoc, Fingerprint}
import com.twitter.util.Future
import com.twitter.botmaker.compiler.types.ScalaType._
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function._

// ====== DO NOT CHANGE BY HAND ======
// The remainder of this file was generated by the codegen script above.
// $ botmaker/botmaker_app/scripts/publisher repl
// > import com.twitter.botmaker._
// > import com.twitter.botmaker.util._
// > FileUtil.writeAllLines( "botmaker/src/scala/com/twitter/botmaker/FunctionUnitGen.scala", FunctionUnit.codeGenPreset)
    """) ::: (0 to 7).toList.flatMap { i =>
      codeGen(i, 0, true) :::
        (0 to 5).toList.flatMap { opt => codeGen(i, opt, false) }
    } ::: codeGen(2, 9) :::
      codeGen(4, 7) :::
      codeGen(11, 1) :::
      codeGen(2, 6) :::
      codeGen(2, 7) :::
      codeGen(2, 8) :::
      codeGen(2, 11) :::
      codeGen(8, 1) :::
      codeGen(10, 1) :::
      codeGen(8, 0) :::
      codeGen(3, 8)
  }

}

abstract class ConfigUnit[E, C <: TwitterRuntime.Thrift[_, _]](implicit c: Manifest[C])
    extends FunctionUnit {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Never
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty

  def apply(context: Context[E], config: C): C

  val thriftClass: Class[_ <: org.apache.thrift.TBase[_, _ <: org.apache.thrift.TFieldIdEnum]] =
    c.runtimeClass.asInstanceOf[Class[_ <: TBase[_, _ <: TFieldIdEnum]]]
  val thriftType: ThriftType = Type.thriftOf(thriftClass)

  val signature: Signature =
    new ASTNode.Signature(Type.pairOf(FunctionUnit.TPA, FunctionUnit.TPB), thriftType)

  val arguments: Seq[String] = thriftType.getFieldNames.asScala map { s =>
    s"$s: ${thriftType.getFieldType(s)}"
  } toSeq

  override val botMakerDoc: BotMakerDoc =
    BotMakerDoc.of(funcName, signature, description, actionLevel, arguments.toArray: _*)

  override def fingerprint(): Fingerprint = Fingerprint.EMPTY

  override def createASTNode(children: ImmutableList[ASTNode[_]]): ASTNode[_] = {
    new ThriftOperator[E, C](
      thriftClass.asInstanceOf[Class[C]],
      funcName,
      children
    ) {
      override protected def apply(context: Context[E], config: C): AnyRef = {
        ConfigUnit.this.apply(context, config)
      }

      override def getSignature: Signature = signature
    }
  }
}
