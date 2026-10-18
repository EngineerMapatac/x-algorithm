package com.twitter.botmaker.rule

import com.twitter.botmaker.Extractor
import com.twitter.botmaker.rule.thriftscala.BotMakerRule

trait ExtractorVisitorUnit {
  def visitor: PartialFunction[Extractor[_], Unit]
  def onRuleInit(rule: BotMakerRule): Unit
  def onRuleExit(): Unit
  def onCompletion(): Unit
}
