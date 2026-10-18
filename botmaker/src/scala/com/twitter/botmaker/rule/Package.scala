package com.twitter.botmaker

import com.google.common.base.Strings
import java.io.PrintWriter
import java.io.StringWriter
import java.util.concurrent.Callable

package object rule {

  type InputProvider = PartialFunction[String, AnyRef]

  implicit def toWrap[A](f: => A): Callable[A] = new Callable[A] {
    override def call(): A = f
  }

  def safe(str: String): String = {
    Strings.nullToEmpty(str)
  }

  def getStackTrace(throwable: Throwable): String = {
    val sw = new StringWriter
    val pw = new PrintWriter(sw, true)
    throwable.printStackTrace(pw)
    sw.getBuffer.toString
  }
}
