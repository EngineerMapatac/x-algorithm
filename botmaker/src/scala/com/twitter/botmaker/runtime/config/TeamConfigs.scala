package com.twitter.botmaker.runtime.config

import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.runtime.thriftjava.TeamConfig
import com.twitter.botmaker.runtime.RuntimeDebugger
import scala.collection.mutable.ArrayBuffer

object Team extends ConfigUnit[TeamConfigs, TeamConfig] {

  override def description: String = "Create a team config."

  override def apply(context: Context[TeamConfigs], config: TeamConfig): TeamConfig = {
    context.getRuntime.addTeam(config)
    config
  }
}

trait TeamConfigs extends ServiceConfigs {

  final private val teams = ArrayBuffer[TeamConfig]()

  final def getTeams: Seq[TeamConfig] = synchronized {
    teams
  }

  final def addTeam(config: TeamConfig): Unit = {
    unique(Team, config.name)
    validateTeamName(config.name)
    this.synchronized {
      teams += config
    }
  }

  override def debug(dbg: RuntimeDebugger): Unit = {

    super.debug(dbg)

    dbg.open("teams", teams) foreach { debugger =>
      teams foreach { config => debugger.info(config.name, config.toString) }
    }
  }
}
