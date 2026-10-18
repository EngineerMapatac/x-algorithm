package com.twitter.botmaker.rule

import com.google.common.base.Strings
import com.google.common.collect.Maps
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.Extractor
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.Parameter
import com.twitter.botmaker.compiler.tuples.StackFrame
import com.twitter.botmaker.compiler.tuples.{Try => TryTuple}
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.DerivedFeatureCall
import com.twitter.botmaker.function.Val
import com.twitter.botmaker.function.Variable
import com.twitter.botmaker.function.With
import com.twitter.botmaker.function.exception.{Try => BMFunctionTry}
import com.twitter.botmaker.function.exception.TryOrElse
import com.twitter.botmaker.function.feature.InputFeature
import com.twitter.botmaker.rule.thriftscala.BotMakerRule
import com.twitter.botmaker.util.TriConsumer
import com.twitter.util.Await
import com.twitter.util.Future
import java.util.function.BiFunction
import java.util.function.Supplier
import java.util.{HashMap => JHashMap}
import java.util.{List => JList}
import scala.collection.JavaConverters._
import scala.collection.mutable

object RuleAst {
  def toStringRepr(ast: ASTNode[_]): String = {

    def s(i: Int): String = toStringRepr(ast.getChildren.get(i))

    ast.getClassName match {
      case "Constant" => ast.getExprText
      case "InputFeature" => ast.getExprText
      case "Variable" => ast.getExprText
      case "Equals" => s"${s(0)} == ${s(1)}"
      case "NotEquals" => s"${s(0)} != ${s(1)}"
      case "LessThan" => s"${s(0)} < ${s(1)}"
      case "LessThanOrEquals" => s"${s(0)} <= ${s(1)}"
      case "GreaterThan" => s"${s(0)} > ${s(1)}"
      case "GreaterThanOrEquals" => s"${s(0)} >= ${s(1)}"
      case "Negate" => s"!${s(0)}"
      case "Conjunction" =>
        (0 until ast.getChildren.size).map(i => s(i)).mkString(" && ")
      case "DisjunctionExtractor" =>
        (0 until ast.getChildren.size).map(i => s(i)).mkString(" || ")
      case "ArithmeticOperation" if ast.getExprText == "Difference" => s"${s(0)} - ${s(1)}"
      case "ArithmeticOperation" if ast.getExprText == "Product" => s"${s(0)} * ${s(1)}"
      case "ArithmeticOperation" if ast.getExprText == "Sum" => s"${s(0)} + ${s(1)}"
      case "ArithmeticOperation" if ast.getExprText == "Quotient" => s"${s(0)} / ${s(1)}"
      case "Mod" => s"${s(0)} % ${s(1)}"
      case "RegexMatch" => s"${s(0)} MATCHES ${s(1)}"
      case "ListOperator" =>
        (0 until ast.getChildren.size).map(i => s(i)).mkString("[", ",", "]")
      case "IfThenElse" if ast.getExprText == "IF" && ast.getChildren.size == 2 =>
        s"if(${s(0)}) {${s(1)}}"
      case "IfThenElse" if ast.getExprText == "IF" && ast.getChildren.size == 3 =>
        s"if(${s(0)}) {${s(1)}} else {${s(2)}}"
      case "IfThenElse" if ast.getExprText == "If" && ast.getChildren.size == 2 =>
        s"If(${s(0)}, ${s(1)})"
      case "IfThenElse" if ast.getExprText == "If" && ast.getChildren.size == 3 =>
        s"If(${s(0)}, ${s(1)}, ${s(2)})"
      case "BlockStatement" =>
        (0 until ast.getChildren.size).map(i => s(i)).mkString("{", ";", "}")
      case "DerivedFeatureCall" if ast.asInstanceOf[DerivedFeatureCall].getVariables.size == 0 =>
        ast.getExprText
      case "DerivedFeatureCall" =>
        val df = ast.asInstanceOf[DerivedFeatureCall]
        val funcName = ast.getExprText
        val args = (0 until df.getVariables.size)
          .map(i => toStringRepr(df.getVariables.get(i).getSecond)).mkString("(", ",", ")")
        s"$funcName$args"
      case _ if Strings.isNullOrEmpty(ast.getExprText) =>
        val funcName = ast.getClassName
        val args = (0 until ast.getChildren.size).map(i => s(i)).mkString("(", ",", ")")
        s"$funcName$args"
      case _ =>
        val funcName = ast.getExprText
        val args = (0 until ast.getChildren.size).map(i => s(i)).mkString("(", ",", ")")
        s"$funcName$args"
    }
  }

  def expand(ast: ASTNode[_]): Seq[ASTNode[_]] = {
    val visited = mutable.Set[ASTNode[_]]()
    expand(ast, visited)
  }

  private def expand(
    ast: ASTNode[_],
    visited: mutable.Set[ASTNode[_]]
  ): Seq[ASTNode[_]] = {
    ast match {
      case x if visited.contains(x) =>
        Nil
      case x: Variable =>
        val ans = expand(x.getChildren.get(0), visited)
        visited += x
        Seq(x) ++ ans
      case x: Parameter =>
        val ans = expand(x.getChildren.get(0), visited)
        visited += x
        Seq(x) ++ ans
      case x: DerivedFeatureCall =>
        visited += x
        Seq(x) ++ x.getVariables.asScala.flatMap(v => expand(v.getSecond, visited)) ++
          x.getChildren.asScala.flatMap(v => expand(v, visited))
      case x: With =>
        visited += x
        Seq(x) ++ x.getVariables.asScala.flatMap(v => expand(v.getSecond, visited)) ++
          x.getChildren.asScala.flatMap(v => expand(v, visited))
      case x =>
        visited += x
        Seq(x) ++ x.getChildren.asScala.flatMap(v => expand(v, visited))
    }
  }
}

class RuleAst(
  val botMakerRule: BotMakerRule,
  val booleanAstRoot: ASTNode[_],
  val actionAstRoot: ASTNode[_]) {

  def id: Long = botMakerRule.id
  def eventType: String = botMakerRule.eventType
  def name: String = botMakerRule.name
  def team: String = botMakerRule.team
  def description: String = botMakerRule.description

  lazy val booleanNodeSet: Set[ASTNode[_]] = RuleAst.expand(booleanAstRoot).toSet
  lazy val actionNodeSet: Set[ASTNode[_]] = RuleAst.expand(actionAstRoot).toSet
  lazy val allNodeSet: Set[ASTNode[_]] = booleanNodeSet union actionNodeSet

  lazy val rootNodeOf: Map[ASTNode[_], ASTNode[_]] =
    (booleanNodeSet.map(_ -> booleanAstRoot) ++ actionNodeSet.map(_ -> actionAstRoot)).toMap

  lazy val parentMap: Map[ASTNode[_], ASTNode[_]] = {
    val map = new JHashMap[ASTNode[_], ASTNode[_]]
    buildParentMap(booleanAstRoot, map)
    buildParentMap(actionAstRoot, map)
    map.asScala.toMap
  }

  def packages: Set[String] = allNodeSet.map(_.getPackageName)

  def derivedFeatures: Seq[DerivedFeatureCall] = {
    filter("DerivedFeatureCall").map(_.asInstanceOf[DerivedFeatureCall])
  }

  def inputFeatures: Seq[InputFeature] = {
    filter("InputFeature").map(_.asInstanceOf[InputFeature])
  }

  def flatten(ast: ASTNode[_]): Seq[ASTNode[_]] = {
    ast match {
      case x: Variable =>
        Seq(x)
      case x: DerivedFeatureCall =>
        x :: x.getVariables.asScala.toList.flatMap(v => flatten(v.getSecond)) :::
          x.getChildren.asScala.toList.flatMap(flatten(_))
      case x: With =>
        x :: x.getVariables.asScala.toList.flatMap(v => flatten(v.getSecond)) :::
          x.getChildren.asScala.toList.flatMap(flatten(_))
      case x =>
        x :: x.getChildren.asScala.toList.flatMap(flatten(_))
    }
  }

  def search(funcName: String): Seq[ASTNode[_]] = {
    filter(funcName).headOption match {
      case Some(node0) =>
        val path = mutable.ListBuffer[ASTNode[_]]()
        path.prepend(node0)
        var node = node0
        while (path.head != booleanAstRoot && path.head != actionAstRoot) {
          node = parentMap(node)
          path.prepend(node)
        }
        path
      case _ => Nil
    }
  }

  def search(node: ASTNode[_]): Seq[ASTNode[_]] = {
    filterFingerprint(node).headOption match {
      case Some(node0) =>
        val path = mutable.ListBuffer[ASTNode[_]]()
        path.prepend(node0)
        var node = node0
        while (path.head != booleanAstRoot && path.head != actionAstRoot) {
          node = parentMap(node)
          path.prepend(node)
        }
        path
      case _ => Nil
    }
  }

  def buildParentMap(ast: ASTNode[_], map: JHashMap[ASTNode[_], ASTNode[_]]): Unit = {
    ast match {
      case _: Variable => ()
      case x: DerivedFeatureCall =>
        x.getVariables.asScala.foreach { v =>
          {
            map.put(v.getSecond, x)
            buildParentMap(v.getSecond, map)
          }
        }
        x.getChildren.asScala.foreach { v =>
          {
            map.put(v, x)
            buildParentMap(v, map)
          }
        }
      case x: With =>
        x.getVariables.asScala.foreach { v =>
          {
            map.put(v.getSecond, x)
            buildParentMap(v.getSecond, map)
          }
        }
        x.getChildren.asScala.foreach { v =>
          {
            map.put(v, x)
            buildParentMap(v, map)
          }
        }
      case x =>
        x.getChildren.asScala.foreach { v =>
          {
            map.put(v, x)
            buildParentMap(v, map)
          }
        }
    }
  }

  def filter(className: String, nodes: Seq[ASTNode[_]] = allNodeSet.toList): Seq[ASTNode[_]] = {
    nodes.filter(_.getClassName == className)
  }

  def filterExprText(
    exprText: String,
    className: String,
    nodes: Seq[ASTNode[_]] = allNodeSet.toList
  ): Seq[ASTNode[_]] = {
    nodes.filter(n => n.getExprText == exprText && n.getClassName == className)
  }

  def filterFingerprint(
    a: ASTNode[_],
    nodes: Seq[ASTNode[_]] = allNodeSet.toList
  ): Seq[ASTNode[_]] = {
    nodes.filter(_.getFingerprint == a.getFingerprint).filter(!_.isInstanceOf[Variable])
  }

  def evaluate(astNode: ASTNode[_]): AnyRef = {
    if (astNode.getFingerprint.scope == ASTNode.CacheLevel.Global.scope) {
      try {
        val extractor = astNode.toExtractor.asInstanceOf[Extractor[RuleAst]]
        val context = new AnalyzeContext(this)
        Await.result(context.run(extractor))
      } catch {
        case t: Throwable =>
          println(("Exception", astNode.getClassName, astNode.getExprText, t))
          astNode.getFingerprint
      }
    } else if (astNode.getFingerprint.scope == ASTNode.CacheLevel.Event.scope) {
      try {
        val extractor = astNode.toExtractor.asInstanceOf[Extractor[RuleAst]]
        val context = new AnalyzeContext(this)
        Await.result(context.run(extractor))
      } catch {
        case t: Throwable =>
          println(("Exception", astNode.getClassName, astNode.getExprText, t))
          astNode.getFingerprint
      }
    } else {
      astNode.getFingerprint
    }
  }

  def getFuncArgs(funcName: String, index: Int): Seq[AnyRef] = {
    filter(funcName).map(gl => evaluate(gl.getChildren.get(index)))
  }

  def getFuncArgPairs(funcName: String, index: Int): Seq[(ASTNode[_], AnyRef)] = {
    filter(funcName).map(gl => gl -> evaluate(gl.getChildren.get(index)))
  }

  def getDerivedFeaturecArgs(funcName: String, index: Int): Seq[AnyRef] = {
    derivedFeatures filter { df =>
      df.getExprText == funcName
    } map { df => evaluate(df.getVariables.get(index).getSecond) }
  }

  def getDerivedFeaturecArgPairs(
    funcName: String,
    index: Int
  ): Seq[(DerivedFeatureCall, AnyRef)] = {
    derivedFeatures filter { df =>
      df.getExprText == funcName
    } map { df => df -> evaluate(df.getVariables.get(index).getSecond) }
  }
}

class AnalyzeContext(record: RuleAst) extends Context[RuleAst] {

  private val scopeVariables = Maps.newConcurrentMap[String, Future[AnyRef]]
  private val variableNotFound = new Object

  override def runtimeSupplier: Supplier[RuleAst] = () => record
  override def getTrackingId: Long =
    throw new Exception("AnalyzeContext cannot get the bot id")
  override def nowMillis(): Long = System.currentTimeMillis
  override def startMillis(): Long =
    throw new Exception("AnalyzeContext cannot get the startMillis")

  override protected val extractorEvaluationFunction: BiFunction[Context[RuleAst], Extractor[
    RuleAst
  ], Future[AnyRef]] =
    (ctx, extractor) => {
      extractor.getAstNode match {
        case x: Variable =>
          ctx.getVariable(x.getVariableText) flatMap {
            case v if v == variableNotFound =>
              val allNodes = if (record.rootNodeOf(x) == record.booleanAstRoot) {
                record.booleanNodeSet
              } else {
                record.actionNodeSet
              }

              val valNodes = allNodes filter { d =>
                d.getFingerprint == x.getFingerprint && !d.isInstanceOf[Variable]
              }

              val gctx = new AnalyzeContext(record)
              valNodes.head match {
                case vn: Val =>
                  gctx.run(vn.getChildren.get(0).toExtractor.asInstanceOf[Extractor[RuleAst]])
                case vn =>
                  gctx.run(vn.toExtractor.asInstanceOf[Extractor[RuleAst]])
              }
            case v => Future.value(v)
          }
        case _: TryOrElse =>
          val tryArgExtractor = extractor.getChildren.get(0).asInstanceOf[Extractor[RuleAst]]
          tryArgExtractor.run(ctx)
        case _: BMFunctionTry =>
          extractor.run(ctx).map {
            case res: TryTuple if res.getExceptionType != null =>
              throw new Exception(
                s"AnalyzeContext cannot properly evaluate Try -" +
                  s" ${res.getExceptionType} - ${res.getExceptionMessage}")
            case goodResponse => goodResponse
          }

        case _ =>
          extractor.run(ctx)
      }
    }

  override val inputFeatureFunction: PartialFunction[String, AnyRef] =
    new PartialFunction[String, AnyRef] {
      override def isDefinedAt(v: String): Boolean =
        throw new Exception(s"Input feature $v not retrievable in AnalyzeContext")

      override def apply(v: String): AnyRef =
        throw new Exception(s"Input feature $v not retrievable in AnalyzeContext")
    }

  override def addOutputFeature(
    namespace: scala.Any,
    name: String,
    `type`: Type,
    value: scala.Any
  ): Unit = ()

  override def reportStat(namespace: scala.Any, statName: String, value: Long): Unit = ()
  override def reportFailure(namespace: scala.Any, category: String, t: Throwable): Unit = ()

  override def reportFunctionEvaluationConsumer: TriConsumer[String, ActionLevel, JList[_]] = {
    (_, _, _) =>
  }

  override def applyActionRateLimit[T](
    context: Context[RuleAst],
    functionName: String,
    actionLevel: ActionLevel,
    func: () => Future[T],
    args: JList[_]
  ): Future[T] = func.apply()

  override def getVariable(variableName: String): Future[AnyRef] = {
    if (scopeVariables.containsKey(variableName)) {
      scopeVariables.get(variableName)
    } else {
      Future.value(variableNotFound)
    }
  }

  override def newVariable(name: String, value: Future[AnyRef]): Unit = {
    scopeVariables.put(name, value)
  }

  override def getStackFrames: JList[StackFrame] = Context.DEFAULT_STACK_FRAMES.get()
}
