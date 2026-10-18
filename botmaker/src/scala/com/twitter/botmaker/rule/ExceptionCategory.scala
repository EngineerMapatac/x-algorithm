package com.twitter.botmaker.rule

sealed trait ExceptionCategory

case object Interruption extends ExceptionCategory

case object Ignorable extends ExceptionCategory

case object NormalException extends ExceptionCategory
