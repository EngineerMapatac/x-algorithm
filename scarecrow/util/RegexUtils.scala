package com.twitter.botmaker.app.scarecrow.util

import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler.RegexCache
import java.util.regex.Pattern
import collection.JavaConverters._

object RegexUtils {
  val UNICODE_SEPARATORS_0 = "\u0300-\u036F"
  val UNICODE_SEPARATORS: String = s"[$UNICODE_SEPARATORS_0]"
  val UNICODE_SEPARATORS_PATTERN: Pattern = Pattern.compile(UNICODE_SEPARATORS)

  val ASCII_SEPARATORS_0 = "\\W_"
  val ALL_SEPARATORS_0: String = s"$UNICODE_SEPARATORS_0$ASCII_SEPARATORS_0"
  val ALL_SEPARATORS: String = s"[$ALL_SEPARATORS_0]"
  val ZERO_OR_MORE_SEPARATORS: String = ALL_SEPARATORS + "*"
  val SEPARATORS_OR_WHITESPACE: String = s"[$ALL_SEPARATORS_0\\s]"
  val ONE_OR_MORE_SEPARATOR_OR_WHITESPACE: String = SEPARATORS_OR_WHITESPACE + "+"
  val SIMILAR_LETTER_GROUPINGS: List[Pattern] = List(
    Pattern.compile("[il1]", Pattern.CASE_INSENSITIVE),
    Pattern.compile("[a@]", Pattern.CASE_INSENSITIVE),
    Pattern.compile("[o0]", Pattern.CASE_INSENSITIVE))

  val SIMILAR_LETTERS_1: (Set[Int], String) =
    Set('i'.toInt, 'l'.toInt, '1'.toInt, 'I'.toInt, 'L'.toInt) -> "[il1]"
  val SIMILAR_LETTERS_2: (Set[Int], String) = Set('a'.toInt, '@'.toInt, 'A'.toInt) -> "[a@]"
  val SIMILAR_LETTERS_3: (Set[Int], String) = Set('o'.toInt, '0'.toInt, 'O'.toInt) -> "[o0]"

  def similarLetters(codePoint: Int): String = {
    codePoint match {
      case _ if SIMILAR_LETTERS_1._1.contains(codePoint) => SIMILAR_LETTERS_1._2
      case _ if SIMILAR_LETTERS_2._1.contains(codePoint) => SIMILAR_LETTERS_2._2
      case _ if SIMILAR_LETTERS_3._1.contains(codePoint) => SIMILAR_LETTERS_3._2
      case _ => new java.lang.StringBuilder(1).appendCodePoint(codePoint).toString
    }
  }

  def separatedLetters(string: String): String = {
    string
      .codePoints()
      .iterator()
      .asScala
      .map((token: Integer) => similarLetters(token))
      .mkString(ZERO_OR_MORE_SEPARATORS)
  }

  def separatedLettersUnicode(string: String): String = {
    string
      .codePoints()
      .iterator()
      .asScala
      .map((token: Integer) => similarLetters(token))
      .mkString(UNICODE_SEPARATORS + "+")
  }

  def eitherSeparatedUnicode(strings: String*): String = "(?:" +
    strings
      .map((token: String) => separatedLettersUnicode(token))
      .mkString("|") + ")"

  def eitherSeparated(strings: String*): String = "(?:" +
    strings
      .map((token: String) => separatedLetters(token))
      .mkString("|") + ")"

  implicit class RichString(val text: String) extends AnyVal {

    def replaceAll(
      context: Context[_],
      pattern: Pattern,
      replacement: String
    ): String = {
      RegexCache.getInterruptibleMatcher(context, pattern, text).replaceAll(replacement)
    }
  }
}
