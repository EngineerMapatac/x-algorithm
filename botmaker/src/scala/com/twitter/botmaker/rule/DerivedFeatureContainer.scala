package com.twitter.botmaker.rule

import java.util.{Collection => JCollection}
import java.util.logging.Logger
import scala.collection.JavaConverters
import com.google.common.base.Optional
import com.google.common.collect.ImmutableList
import com.google.common.collect.ImmutableSet
import org.antlr.runtime.tree.Tree
import com.twitter.botmaker.compiler.DerivedFeature
import com.twitter.botmaker.compiler.DerivedFeatureProvider
import com.twitter.botmaker.compiler.Fingerprint
import com.twitter.botmaker.compiler.Parser
import com.twitter.botmaker.compiler.tuples.Tuple3
import com.twitter.botmaker.rule.DerivedFeatureContainer.Node
import com.twitter.botmaker.rule.thriftscala.BotMakerDerivedFeature

class DerivedFeatureContainer private (private val nodes: Map[String, Node] = Map[String, Node]())
    extends DerivedFeatureProvider {

  val fingerprints: Set[Fingerprint] = nodes.values.map(_.getFingerprint).toSet

  override def get(derivedFeatureName: String): Optional[DerivedFeature] = {
    if (nodes.contains(derivedFeatureName)) {
      Optional.of(nodes(derivedFeatureName))
    } else {
      Optional.absent()
    }
  }

  override def containsFingerprint(fingerprint: Fingerprint): Boolean = {
    fingerprints.contains(fingerprint)
  }

  override def all(): JCollection[DerivedFeature] = {
    JavaConverters.asJavaCollection(nodes.values)
  }
}

object DerivedFeatureContainer {
  private val logger: Logger = Logger.getLogger(getClass.getName)

  case class Node(data: BotMakerDerivedFeature) extends DerivedFeature {

    private lazy val parsed = {
      Parser.createParseParamsTree(data.featureExpression)
    }

    private lazy val hashTags = DerivedFeature.extractHashTags(data.description)

    override lazy val getFingerprint: Fingerprint = Fingerprint.of(data)

    override def getName: String = data.name

    override def getDescription: String = data.description

    override def getExpr: String = data.featureExpression

    override def isLogged: Boolean = data.logged.getOrElse(false)

    override def getTree: Tree = parsed.getFirst

    override def getParams: ImmutableList[Tuple3[Tree, String, Optional[Tree]]] = parsed.getSecond

    override def getReturnType: Optional[Tree] = parsed.getThird

    override def getFixedReturnType: String = data.returnType

    override def isOverrideFunction: Boolean = false

    override def equals(obj: scala.Any): Boolean = obj match {
      case x: Node => x.data.equals(data)
      case x: BotMakerDerivedFeature => x.equals(data)
      case _ => false
    }

    override def getHashTags: ImmutableSet[String] = hashTags
  }

  val empty: DerivedFeatureContainer = new DerivedFeatureContainer

  def apply(botMakerDerivedFeatures: Seq[BotMakerDerivedFeature]): DerivedFeatureContainer = {
    apply(empty, botMakerDerivedFeatures)
  }

  def apply(
    container: DerivedFeatureContainer,
    botMakerDerivedFeatures: Seq[BotMakerDerivedFeature]
  ): DerivedFeatureContainer = {
    val nodes: Map[String, Node] = botMakerDerivedFeatures.map { f: BotMakerDerivedFeature =>
      if (container.nodes.contains(f.name) && areEquivalent(container.nodes(f.name).data, f)) {
        f.name -> container.nodes(f.name)
      } else {
        if (container.nodes.contains(f.name)) {
          logger.info(s"cached DF ${container.nodes(f.name).data} is not equivalent to $f")
        } else {
          logger.info(s"DF ${f.name} not cached")
        }
        logger.info(s"putting DF ${f.name} into container...")
        val n = Node(f)
        logger.info(s"finished putting DF ${f.name} into container")
        f.name -> n
      }
    }.toMap

    new DerivedFeatureContainer(nodes)
  }

  private def areEquivalent(df1: BotMakerDerivedFeature, df2: BotMakerDerivedFeature): Boolean = {
    df1.name == df2.name &&
    df1.featureExpression.replaceAll("\\n\\s+", "\n") == df2.featureExpression
      .replaceAll("\\n\\s+", "\n") &&
    df1.returnType == df2.returnType &&
    df1.logged == df2.logged
  }
}
