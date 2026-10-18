package com.twitter.botmaker.runtime.config

import scala.collection.mutable.ArrayBuffer
import scala.reflect.ClassTag
import scala.collection.JavaConverters._
import com.twitter.botmaker.ConfigUnit
import com.twitter.botmaker.Context
import com.twitter.botmaker.compiler._
import com.twitter.botmaker.runtime.ConfigFailure
import com.twitter.botmaker.runtime.ServiceContainer
import com.twitter.botmaker.runtime.thriftjava.ThriftEndpointConfig
import com.twitter.botmaker.runtime.thriftjava.ThriftMethodConfig
import com.twitter.util.Duration

object ThriftEndpoint extends ConfigUnit[ThriftEndpointConfigs, ThriftEndpointConfig] {

  override def description: String = "Create a thrift endpoint."

  override def apply(
    context: Context[ThriftEndpointConfigs],
    config: ThriftEndpointConfig
  ): ThriftEndpointConfig = {
    context.getRuntime.addThriftEndpoint(config)
    config
  }
}

object ThriftMethod extends ConfigUnit[ThriftEndpointConfigs, ThriftMethodConfig] {

  override def description: String = "Config a thrift method."

  override def apply(
    context: Context[ThriftEndpointConfigs],
    config: ThriftMethodConfig
  ): ThriftMethodConfig = {
    config
  }
}

trait ThriftEndpointConfigs extends ServiceConfigs {

  private final val thriftEndpoints = ArrayBuffer[ThriftEndpointConfig]()

  final def getThriftEndpoints: Seq[ThriftEndpointConfig] = thriftEndpoints.toSeq

  final def getThriftEndpoint(endpoint: String): Option[ThriftEndpointConfig] =
    thriftEndpoints.filter(_.endpoint == endpoint).headOption

  final def addThriftEndpoint(config: ThriftEndpointConfig): Unit = {

    unique(Endpoint, config.endpoint)

    validateEndpoint(config.endpoint)
    if (config.isSetSessionTimeoutMillis && config.sessionTimeoutMillis <= 0) {
      throw ConfigFailure(
        s"invalid session timeout value ${config.sessionTimeoutMillis} in thrift endpoint ${config.endpoint}"
      )
    }

    if (config.isSetRequestTimeoutMillis && config.requestTimeoutMillis <= 0) {
      throw ConfigFailure(
        s"invalid request timeout value ${config.requestTimeoutMillis} in thrift endpoint ${config.endpoint}"
      )
    }

    config.methods.asScala foreach { tm =>
      unique(FuncName, config.endpoint, tm.funcName)
      validateFuncName(tm.funcName)
      if (tm.isSetTimeoutMillis && tm.timeoutMillis <= 0) {
        throw ConfigFailure(
          s"invalid timeout value ${tm.timeoutMillis} in thrift method ${tm.funcName} of ${config.endpoint}"
        )
      }

    }

    thriftEndpoints.append(config)
  }

  protected def mux[Iface: ClassTag](
    endpoint: String,
    servicePath: String,
    clientId: String,
    sessionTimeout: Duration,
    requestTimeout: Duration,
    retries: Int
  ): Unit = {
    val ctag = implicitly[ClassTag[Iface]]
    val config = new ThriftEndpointConfig()
      .setEndpoint(endpoint)
      .setServicePath(servicePath)
      .setClientId(clientId)
      .setThriftClass(ctag.runtimeClass.getName)
      .setSessionTimeoutMillis(sessionTimeout.inMillis)
      .setRequestTimeoutMillis(requestTimeout.inMillis)

    validateEndpoint(config.endpoint)
    thriftEndpoints.append(config)
  }

  override def compilerBuilder[E <: ServiceContainer: ClassTag]: CompilerBuilder[E] = {
    val builder = super.compilerBuilder[E]

    thriftEndpoints foreach { te =>
      te.methods.asScala foreach { tm =>
        val p = com.twitter.botmaker.runtime.ThriftMethod(te, tm)
        val cu = p.toCompile
        builder.addCompilerUnit(cu)
      }
    }

    builder
  }
}
