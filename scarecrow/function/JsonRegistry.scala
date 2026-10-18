package com.twitter.botmaker.app.scarecrow.function

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.scala.DefaultScalaModule
import com.twitter.botmaker.ASTNode
import com.twitter.botmaker.Context
import com.twitter.botmaker.DownstreamService
import com.twitter.botmaker.FunctionUnit1
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.app.scarecrow.function.JsonRegistryMapper.parser
import com.twitter.botmaker.compiler.ActionLevel
import java.util.{List => JList}
import java.util.{Map => JMap}

object JsonParse extends FunctionUnit1[ScarecrowRuntime, Option[String], Option[AnyRef]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Global

  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION

  override def downstreams: Set[DownstreamService] = Set.empty

  override def description: String =
    """Given a string encoded json, parse and return the objects it represents. For instance, 
      |if the input was of a json object with array values, the return would be a Map with String 
      |keys and List values. An input value like "foo" will be parsed correctly as well into the 
      |three-character String 'foo'""".stripMargin

  override def arguments: Seq[String] = Seq("String encoded JSON")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    jsonStr: Option[String]
  ): Option[AnyRef] = jsonStr match {
    case Some(str) =>
      val node = parser.readTree(str)
      if (node.isObject) {
        Some(parser.treeToValue(node, classOf[JMap[_, _]]))
      } else if (node.isArray) {
        Some(parser.treeToValue(node, classOf[JList[_]]))
      } else {
        toBasicValue(node)
      }
    case _ => None
  }

  private def toBasicValue(node: JsonNode): Option[AnyRef] = {
    if (node.isBoolean) {
      Some(Boolean.box(node.asBoolean))
    } else if (node.isIntegralNumber) {
      Some(Long.box(node.asLong))
    } else if (node.isFloatingPointNumber) {
      Some(Double.box(node.asDouble))
    } else if (node.isTextual) {
      Some(node.asText)
    } else if (node.isNull) {
      None
    } else {
      throw new RuntimeException(s"unhandled value node type ${node.getClass}")
    }
  }
}

private[function] object JsonRegistryMapper {
  final val parser = new ObjectMapper()
    .configure(DeserializationFeature.USE_LONG_FOR_INTS, true)

  final val serializer = new ObjectMapper()
    .registerModule(new DefaultScalaModule)
}
