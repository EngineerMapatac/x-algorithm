package com.twitter.botmaker.rule.function

import java.util.concurrent.atomic.AtomicInteger
import java.util.{Set => JSet}
import com.google.common.collect.ImmutableSet
import com.twitter.botmaker._
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.types.Type
import com.twitter.botmaker.function.feature.OutputFeature
import com.twitter.botmaker.rule.ResponseOutput
import com.twitter.botmaker.runtime.TwitterRuntime
import com.twitter.botmaker.thriftscala.{ResponseCode => ScalaResponseCode}
import com.twitter.bouncer.thriftjava.Bounce
import com.twitter.bouncer.thriftjava.{BounceExperience => TBounceExperience}
import com.twitter.service.gen.scarecrow.TieredAction
import com.twitter.service.gen.scarecrow.TieredActionResult
import com.twitter.util.Try
import scala.collection.JavaConverters._

object ResponseCodes {

  val BounceErrorMessage = "bounceErrorMessage"
  val BounceExperience = "bounceExperience"
  val BounceLocation = "bounceLocation"

  val RedirectLocation = "redirectLocation"
  val HttpResponseHeaders = "httpResponseHeaders"
  val HttpStatusCode = "httpStatusCode"

  val None: JSet[ResponseCode] = ImmutableSet.of(ResponseCode.NONE)

  private[this] val nextPrecedence = new AtomicInteger
  val ResponseCodePrecedence: Map[String, Int] = Map(
    ResponseCode.APP_MUZZLED.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.WHITE_LISTED.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.ACTION_SUCCESS.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.ERROR.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.TAKE_NO_ACTION.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.URL_SPAM.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.DENY.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.BOUNCE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.CHECK_BOUNCE_EXPERIMENT.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.SUPPRESS_NOTIFICATIONS.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.REVIEW.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.RATE_LIMIT.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.PHONE_VERIFY_SYNC.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.SUSPEND.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.PASSWORD_RESET.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.READ_ONLY.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.SILENT_FAIL.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.TRY_LATER.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.THROW_CAPTCHA.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.RETYPE_PHONE_NUMBER.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.RETYPE_EMAIL.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.VERIFY_TEMPORARY_PASSWORD.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.RETYPE_SCREEN_NAME.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.CHALLENGE_LOGIN.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.CHALLENGE_LOGIN_PHONE_OWNERSHIP.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.HIDE_NOTIFICATION.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.USER_VOTE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.DENY_BY_IPI_POLICY.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.BAD_REQUEST.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.NOT_FOUND.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.UNAUTHORIZED.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.FORBIDDEN.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.REDIRECT.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.HTTP_RESPONSE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.SERVICE_UNAVAILABLE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.DECORATE_RESPONSE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.BACKOFF_DECORATION.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.BOUNCE_REDIRECT.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.NONE.name -> nextPrecedence.incrementAndGet(),
    ResponseCode.BLACK_LISTED.name -> nextPrecedence.incrementAndGet()
  )

  def toTiredAction(eventType: String, response: BotMakerResponse): TieredAction = {
    val actionResult = response.finalResponse.code match {
      case ResponseCode.APP_MUZZLED => TieredActionResult.APP_MUZZLED
      case ResponseCode.DENY => TieredActionResult.DENY
      case ResponseCode.PHONE_VERIFY_SYNC => TieredActionResult.PHONE_VERIFY
      case ResponseCode.SILENT_FAIL => TieredActionResult.SILENT_FAIL
      case ResponseCode.TRY_LATER => TieredActionResult.TRY_LATER
      case ResponseCode.HIDE_NOTIFICATION => TieredActionResult.HIDE_NOTIFICATION
      case ResponseCode.THROW_CAPTCHA => TieredActionResult.CAPTCHA
      case ResponseCode.RETYPE_PHONE_NUMBER => TieredActionResult.RETYPE_PHONE_NUMBER
      case ResponseCode.RETYPE_EMAIL => TieredActionResult.RETYPE_EMAIL
      case ResponseCode.VERIFY_TEMPORARY_PASSWORD =>
        TieredActionResult.VERIFY_TEMPORARY_PASSWORD
      case ResponseCode.RETYPE_SCREEN_NAME =>
        TieredActionResult.RETYPE_SCREEN_NAME
      case ResponseCode.CHALLENGE_LOGIN => TieredActionResult.CHALLENGE_LOGIN
      case ResponseCode.USER_VOTE => TieredActionResult.USER_VOTE
      case ResponseCode.READ_ONLY => TieredActionResult.DENY
      case ResponseCode.SUSPEND => TieredActionResult.DENY
      case ResponseCode.PASSWORD_RESET => TieredActionResult.DENY
      case ResponseCode.RATE_LIMIT => TieredActionResult.RATE_LIMIT
      case ResponseCode.BOUNCE => TieredActionResult.BOUNCE
      case ResponseCode.SUPPRESS_NOTIFICATIONS => TieredActionResult.SUPPRESS_NOTIFICATIONS
      case ResponseCode.ACTION_SUCCESS => TieredActionResult.ACTION_SUCCESS
      case ResponseCode.CHECK_BOUNCE_EXPERIMENT => TieredActionResult.CHECK_BOUNCE_EXPERIMENT
      case ResponseCode.ERROR => TieredActionResult.ERROR
      case ResponseCode.TAKE_NO_ACTION => TieredActionResult.TAKE_NO_ACTION
      case ResponseCode.DENY_BY_IPI_POLICY => TieredActionResult.DENY_BY_IPI_POLICY
      case ResponseCode.BAD_REQUEST => TieredActionResult.BAD_REQUEST
      case ResponseCode.NOT_FOUND => TieredActionResult.NOT_FOUND
      case ResponseCode.UNAUTHORIZED => TieredActionResult.UNAUTHORIZED
      case ResponseCode.FORBIDDEN => TieredActionResult.FORBIDDEN
      case ResponseCode.REDIRECT => TieredActionResult.REDIRECT
      case ResponseCode.HTTP_RESPONSE => TieredActionResult.HTTP_RESPONSE
      case ResponseCode.URL_SPAM => TieredActionResult.URL_SPAM
      case ResponseCode.SERVICE_UNAVAILABLE => TieredActionResult.SERVICE_UNAVAILABLE
      case ResponseCode.DECORATE_RESPONSE => TieredActionResult.DECORATE_RESPONSE
      case ResponseCode.BACKOFF_DECORATION => TieredActionResult.BACKOFF_DECORATION
      case ResponseCode.BOUNCE_REDIRECT => TieredActionResult.BOUNCE_REDIRECT
      case _ if eventType == "login_write" => TieredActionResult.NO_CHALLENGE_LOGIN
      case _ => TieredActionResult.NOT_SPAM
    }
    val tieredAction = new TieredAction()
      .setResultCode(
        actionResult
      )
      .setBotMakerResponse(response)

    if (tieredAction.resultCode == TieredActionResult.BOUNCE) {
      tieredAction.setBounce(ResponseCodes.extractBounce(response))
    }

    tieredAction
  }

  def extractBounce(response: BotMakerResponse): Bounce = {

    val bounce: Bounce = new Bounce().setExperience(TBounceExperience.FULL_OPTIONAL)
    if (response.isSetOutputFeatures) {
      val features = response.getOutputFeatures
      if (features.containsKey(BounceExperience)) {
        val exp: String = features.get(BounceExperience).getFeatureValue.getStrValue
        Try {
          bounce.setExperience(TBounceExperience.valueOf(exp))
        }
      }
      if (features.containsKey(BounceLocation)) {
        val fv = features.get(BounceLocation).getFeatureValue
        bounce.setLocation(fv.getStrValue)
      }
      if (features.containsKey(BounceErrorMessage)) {
        val fv = features.get(BounceErrorMessage).getFeatureValue
        bounce.setError_message(fv.getStrValue)
      }
    }
    bounce
  }

  def resolveFinalResponseCode(ruleResponses: Map[Long, ResponseCode]): (Long, ResponseCode) = {
    if (ruleResponses.isEmpty) {
      0L -> ResponseCode.NONE
    } else {
      ruleResponses minBy {
        case (_, code) =>
          ResponseCodePrecedence.getOrElse(code.name, ResponseCodePrecedence.size + 1)
      }
    }
  }

  def resolveResponseCode(responseCodes: scala.collection.Set[ResponseCode]): ResponseCode = {
    if (responseCodes.isEmpty) {
      ResponseCode.NONE
    } else {
      responseCodes minBy { response: ResponseCode =>
        ResponseCodePrecedence.getOrElse(response.name, ResponseCodePrecedence.size + 1)
      }
    }
  }

  private val NoneResponse = new RuleResponse(ResponseCode.NONE, 0)

  def resolveFinalResponse(ruleResponses: scala.collection.Set[RuleResponse]): RuleResponse = {
    if (ruleResponses.isEmpty) {
      NoneResponse
    } else {
      ruleResponses minBy (response =>
        ResponseCodePrecedence.getOrElse(response.code.name, ResponseCodePrecedence.size + 1))
    }
  }

  def toScala(responseCode: ResponseCode): ScalaResponseCode = {
    ScalaResponseCode.apply(responseCode.getValue)
  }

  def toScala(responseCodeName: String): ScalaResponseCode = {
    ScalaResponseCode.valueOf(responseCodeName).get
  }

  def apply(respondCodes: ResponseCode*): ImmutableSet[ResponseCode] = {
    ImmutableSet.copyOf(respondCodes.toArray)
  }
}

class SimpleResponseCode(responseCode: ResponseCode, override val actionLevel: ActionLevel)
    extends FunctionUnit0[TwitterRuntime, java.util.Set[ResponseCode]] {

  private val responseCodeSet = ImmutableSet.of(responseCode)

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = s"Set the response code as $responseCode"
  override def arguments: Seq[String] = Nil
  override def evaluate(context: Context[TwitterRuntime]): JSet[ResponseCode] = {
    context.addOutputFeature(
      ResponseOutput,
      ResponseOutput.ResponseCode,
      Type.OBJECT,
      responseCode
    )
    responseCodeSet
  }
}

object None
    extends SimpleResponseCode(ResponseCode.NONE, ActionLevel.RESPONSE_CODE_NO_CHANGE_ACTION)

object BadRequest
    extends SimpleResponseCode(
      ResponseCode.BAD_REQUEST,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object Deny
    extends SimpleResponseCode(ResponseCode.DENY, ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object Error
    extends SimpleResponseCode(ResponseCode.ERROR, ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object Forbidden
    extends SimpleResponseCode(
      ResponseCode.FORBIDDEN,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object NotFound
    extends SimpleResponseCode(
      ResponseCode.NOT_FOUND,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object RateLimit
    extends SimpleResponseCode(
      ResponseCode.RATE_LIMIT,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object Unauthorized
    extends SimpleResponseCode(
      ResponseCode.UNAUTHORIZED,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object ServiceUnavailable
    extends SimpleResponseCode(
      ResponseCode.SERVICE_UNAVAILABLE,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object BackoffDecoration
    extends SimpleResponseCode(
      ResponseCode.BACKOFF_DECORATION,
      ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION)

object Redirect extends FunctionUnit1[TwitterRuntime, String, JSet[ResponseCode]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Returns the Redirect ResponseCode."
  override def arguments: Seq[String] =
    Seq[String]("Redirect Location")
  override def examples: Seq[String] = Seq[String](
    """Redirect("/foo/bar")"""
  )

  override def evaluate(context: Context[TwitterRuntime], location: String): JSet[ResponseCode] = {
    val responseCode = ResponseCode.REDIRECT
    context.addOutputFeature(
      OutputFeature.NAMESPACE,
      ResponseCodes.RedirectLocation,
      Type.STRING,
      location
    )
    context.addOutputFeature(
      ResponseOutput,
      ResponseOutput.ResponseCode,
      Type.OBJECT,
      responseCode
    )
    ImmutableSet.of(responseCode)
  }
}

object HttpResponse
    extends FunctionUnit2[TwitterRuntime, Int, Map[String, String], JSet[ResponseCode]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description = "Returns a general HTTP response."
  override def arguments: Seq[String] =
    Seq[String]("Status Code", "HTTP Response Headers")
  override def examples: Seq[String] = Seq[String](
    """HttpResponse(503,
      |  Map(
      |    "Transfer-Encoding": "chunked",
      |    "Connection": "Keep-Alive"
      |  ))""".stripMargin
  )

  override def evaluate(
    context: Context[TwitterRuntime],
    statusCode: Int,
    headers: Map[String, String]
  ): JSet[ResponseCode] = {
    val responseCode = ResponseCode.HTTP_RESPONSE
    context.addOutputFeature(
      OutputFeature.NAMESPACE,
      ResponseCodes.HttpStatusCode,
      Type.INTEGER,
      statusCode
    )
    context.addOutputFeature(
      OutputFeature.NAMESPACE,
      ResponseCodes.HttpResponseHeaders,
      Type.mapOf(Type.STRING, Type.STRING),
      headers.asJava
    )
    context.addOutputFeature(
      ResponseOutput,
      ResponseOutput.ResponseCode,
      Type.OBJECT,
      responseCode
    )
    ImmutableSet.of(responseCode)
  }
}

object DecorateResponse
    extends FunctionUnit1[TwitterRuntime, Map[String, String], JSet[ResponseCode]] {

  override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.RESPONSE_CODE_NO_CHANGE_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String = "Returns a map of response decoration headers."
  override def arguments: Seq[String] = Seq("HTTP Response Headers")
  override def examples: Seq[String] = Seq(
    """DecorateResponse(
      |  Map(
      |    "Transfer-Encoding": "chunked",
      |    "Connection": "Keep-Alive"
      |  ))""".stripMargin
  )

  override def evaluate(
    context: Context[TwitterRuntime],
    headers: Map[String, String]
  ): JSet[ResponseCode] = {
    val responseCode = ResponseCode.DECORATE_RESPONSE
    context.addOutputFeature(
      OutputFeature.NAMESPACE,
      ResponseCodes.HttpResponseHeaders,
      Type.mapOf(Type.STRING, Type.STRING),
      headers.asJava
    )
    context.addOutputFeature(
      ResponseOutput,
      ResponseOutput.ResponseCode,
      Type.OBJECT,
      responseCode
    )
    ImmutableSet.of(responseCode)
  }

  object BounceRedirect
      extends FunctionUnit3[TwitterRuntime, String, String, String, JSet[ResponseCode]] {

    override def cacheLevel: ASTNode.CacheLevel = ASTNode.CacheLevel.Event
    override def actionLevel: ActionLevel = ActionLevel.RESPONSE_CODE_CHANGE_BEHAVIOR_ACTION
    override def downstreams: Set[DownstreamService] = Set.empty
    override def description =
      "Returns the Bounce ResponseCode and performs (forced) redirect to Bounce location"
    override def arguments: Seq[String] =
      Seq[String]("Bounce Experience", "Bounce Location", "Error Message for 3rd Party Client")
    override def examples: Seq[String] = Seq[String](
      """BounceRedirect("FULL_REQUIRED", "/foo/bar", "some message")"""
    )

    override def evaluate(
      context: Context[TwitterRuntime],
      experience: String,
      location: String,
      errorMessage: String
    ): JSet[ResponseCode] = {
      val responseCode = ResponseCode.BOUNCE
      context.addOutputFeature(
        OutputFeature.NAMESPACE,
        ResponseCodes.BounceExperience,
        Type.STRING,
        experience
      )
      context.addOutputFeature(
        OutputFeature.NAMESPACE,
        ResponseCodes.BounceLocation,
        Type.STRING,
        location
      )
      context.addOutputFeature(
        OutputFeature.NAMESPACE,
        ResponseCodes.BounceErrorMessage,
        Type.STRING,
        errorMessage
      )
      context.addOutputFeature(
        ResponseOutput,
        ResponseOutput.ResponseCode,
        Type.OBJECT,
        responseCode
      )
      ImmutableSet.of(responseCode)
    }
  }
}
