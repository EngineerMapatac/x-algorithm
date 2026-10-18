package com.twitter.botmaker.runtime

case class ConfigFailure(msg: String) extends Throwable(msg)
