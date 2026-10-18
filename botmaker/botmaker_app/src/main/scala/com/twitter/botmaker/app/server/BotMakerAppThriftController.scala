package com.twitter.botmaker.app.server

import javax.inject.Inject
import javax.inject.Singleton
import com.twitter.botmaker.app.module.BotMakerApp
import com.twitter.botmaker.app.module.BotMakerAppController
import com.twitter.botmaker.app.thriftjava.AppService

@Singleton
class AppThriftController @Inject() (val botmaker: BotMakerApp)
    extends BotMakerAppController
    with AppService.ServiceIface
