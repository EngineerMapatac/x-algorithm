package com.twitter.botmaker.app.util

import org.json.simple.JSONValue
import scala.collection.immutable.Map

object JsonUtils {

  def buildJsonString(map: Map[String, Any]): String = {
    "{" +
      map.toSeq
        .map {
          case (key, value) =>
            val escapedKey = JSONValue.escape(key)
            value match {
              case map: Map[_, _] =>
                s"""
                 |"$escapedKey" : ${buildJsonString(value.asInstanceOf[Map[String, Any]])}
             """.stripMargin.trim
              case seq: Seq[_] =>
                s"""
                 |"$escapedKey" : ${buildJsonString(seq)}
             """.stripMargin.trim
              case string: String =>
                s"""
                 |"$escapedKey" : "${JSONValue.escape(string)}"
             """.stripMargin.trim
              case _ =>
                s"""
                 |"$escapedKey" : $value
             """.stripMargin.trim
            }
        }.mkString(", ") +
      "}"
  }

  def buildJsonString(seq: Seq[Any]): String = {
    "[" +
      seq
        .map {
          case item: Map[_, _] =>
            buildJsonString(item.asInstanceOf[Map[String, Any]])
          case item: Seq[_] =>
            buildJsonString(item.asInstanceOf[Seq[Any]])
          case item: String =>
            s"""
             |"${JSONValue.escape(item)}"
         """.stripMargin.trim
          case item =>
            s"$item"
        }.mkString(", ") +
      "]"
  }
}
