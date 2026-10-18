package com.twitter.botmaker.app.scarecrow

import com.twitter.botmaker.app.scarecrow.legacy.ScarecrowModule
import com.twitter.inject.TwitterModule
import com.twitter.thriftwebforms.MethodOptions

object BotMakerAppMain extends ScarecrowAppServer(isAdmin = false)
object BotMakerAdminMain extends ScarecrowAppServer(isAdmin = true)

class ScarecrowAppServer(isAdmin: Boolean = false)
    extends com.twitter.botmaker.app.server.BotMakerAppServer {

  override val modules: Seq[TwitterModule] = {
    val module = new Module(isAdmin = isAdmin)
    super.modules ++ Seq(
      module,
      new ScarecrowModule()
    )
  }

  override def thriftWebFormsAccess =
    MethodOptions.Access.ByLdapGroup(Seq("health-core-infra-team"))

  override protected def postWarmup(): Unit = {
    val initializer = injector.instance[BotmakerInitializer]
    initializer.init()
    val scarecrowBotMaker = injector.instance[ScarecrowBotMaker]
    super.postWarmup(scarecrowBotMaker)
    super.postWarmup()
  }

}
