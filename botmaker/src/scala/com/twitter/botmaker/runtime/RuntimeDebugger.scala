package com.twitter.botmaker.runtime

import scala.collection.mutable

trait Debuggable {
  def debug(dbg: RuntimeDebugger): Unit
}

case class RuntimeDebugger(
  cmd: RuntimeDebugger.Command,
  path: List[String],
  private val pos: List[String] = Nil,
  private val collector: mutable.Map[List[String], String] = mutable.Map.empty) {

  def open(name: String, target: Any): Option[RuntimeDebugger] = {

    path match {
      case h :: t if h == name =>
        Some(RuntimeDebugger(cmd, t, name :: pos, collector))
      case "*" :: Nil =>
        Some(RuntimeDebugger(cmd, path, name :: pos, collector))
      case h :: t =>
        None
      case Nil if cmd.isInstanceOf[RuntimeDebugger.Info] =>
        info(name, target.getClass.getSimpleName)
        None
      case _ =>
        None
    }
  }

  def info(key: String, value: String): Any = {
    path match {
      case h :: _ if h == key =>
        collector += (key :: pos) -> value
      case "*" :: Nil =>
        collector += (key :: pos) -> value
      case h :: _ =>
        ()
      case _ =>
        collector += (key :: pos) -> value
    }
  }

  def infos(pairs: (String, String)*): Unit = pairs foreach {
    case (key, value) =>
      info(key, value)
  }

  def toMap: Map[String, String] = {
    collector.toMap map {
      case (k, v) => k.reverse.mkString(".") -> v
    }
  }
}

object RuntimeDebugger {

  trait Command

  case class Info() extends Command

  def infos(runtime: TwitterRuntime, path: String*): Map[String, String] = {
    infos(runtime, path.toList)
  }

  def infos(runtime: TwitterRuntime, path: List[String]): Map[String, String] = {
    val dbg = RuntimeDebugger(Info(), path.toList)
    runtime.debug(dbg)
    dbg.toMap
  }

}
