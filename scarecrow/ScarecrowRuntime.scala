package com.twitter.botmaker.app.scarecrow

import com.twitter.botmaker.app.runtime.StdBotMakerAppRuntime
import com.twitter.botmaker.app.scarecrow.config.ScarecrowConfigs
import com.twitter.botmaker.app.scarecrow.function.AdminFetcher
import com.twitter.botmaker.app.scarecrow.function.Fetcher20
import com.twitter.botmaker.app.scarecrow.legacy._
import com.twitter.botmaker.app.scarecrow.migrate.MigrateContext20
import com.twitter.botmaker.fetcher.Fetcher10
import com.twitter.botmaker.runtime._
import java.util.Collections
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import com.twitter.botmaker.Context
import com.twitter.botmaker.MigrateContext
import com.twitter.spam.decider.Decider
import java.util.concurrent.atomic.AtomicReference

class StaticContext {
  private val _verdictSideEffects: AtomicReference[JMap[String, JSet[Long]]] =
    StaticContext.EmptyVerdictSideEffects

  def verdictSideEffects_=(map: JMap[String, JSet[Long]]): Unit = {
    this._verdictSideEffects.set(map)
  }

  def verdictSideEffects: JMap[String, JSet[Long]] = {
    this._verdictSideEffects.get()
  }
}

object StaticContext {
  val EmptyVerdictSideEffects =
    new AtomicReference[JMap[String, JSet[Long]]](Collections.emptyMap())
}

case class ScarecrowRuntime(
  override val botmaker: ScarecrowBotMaker,
  fetcher10: Fetcher10,
  fetcher20: Fetcher20 = Fetcher20.empty,
  adminFetcher: AdminFetcher = AdminFetcher.empty,
  protected val models: ModelRuntime = ModelRuntime.empty,
  override val registry: Seq[ServiceRegistry[Any]] = Nil,
  staticContext: StaticContext = new StaticContext)
    extends StdBotMakerAppRuntime[ScarecrowConfigs](botmaker, registry)
    with ModelScarecrowRuntime {

  override def debug(dbg: RuntimeDebugger): Unit = {
    super.debug(dbg)
  }

  def botmakerDecider: Decider = fetcher10.getDecider

  def systemConfigs: ScarecrowConfigs = botmaker.state.configContainer.systemConfigs

  def getMigrateContext(context: Context[ScarecrowRuntime]): MigrateContext = {
    MigrateContext20(context)
  }
}
