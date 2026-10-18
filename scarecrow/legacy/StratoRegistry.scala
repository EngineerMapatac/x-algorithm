package com.twitter.botmaker.app.scarecrow.legacy

import com.google.common.collect.ImmutableSet
import com.twitter.abuse.detection.scoring.thriftscala.UserHealthModel
import com.twitter.botmaker._
import com.twitter.botmaker.ASTNode.CacheLevel
import com.twitter.botmaker.app.scarecrow.ScarecrowRuntime
import com.twitter.botmaker.compiler.ActionLevel
import com.twitter.botmaker.compiler.types.Type
import com.twitter.finagle.http.Method
import com.twitter.finagle.http.Request
import com.twitter.finagle.http.Version
import com.twitter.health.platform_manipulation.user_cred_v2.thriftscala.UserCredV2Scores
import com.twitter.hss.api.thriftscala.UserHealthSignal
import com.twitter.hss.api.thriftscala.SignalValue
import com.twitter.spam.rtf.thriftjava.{SafetyLabel => JavaSafetyLabel}
import com.twitter.spam.rtf.thriftjava.{SafetyLabelType => JavaSafetyLabelType}
import com.twitter.util.Future
import java.util.{Map => JMap}
import java.util.{Set => JSet}
import com.twitter.io.Buf
import com.twitter.io.Reader
import scala.collection.JavaConverters._

object DoStratoColumnOp
    extends FunctionUnit4[ScarecrowRuntime, String, String, String, String, Future[String]] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.PROD_TANGIBLE_IMPACT_ACTION
  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.StratoUserNsfaScore)
  override def description = "Given space, path, op and args JSON string, do the strato column op"
  override def arguments: Seq[String] = Seq("space", "path", "op", "args JSON string")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    space: String,
    path: String,
    op: String,
    argsJsonString: String
  ): Future[String] = {
    val pathAndSpace = if (space.isEmpty) {
      path
    } else {
      s"$path.$space"
    }
    context.getRuntime.fetcher20.stratoFetcher20
      .doApiRequest(
        Request(
          Version.Http11,
          Method.Post,
          s"/op/$op/$pathAndSpace",
          Reader.fromBuf(Buf.Utf8(argsJsonString)))).map(_.contentString)
  }
}

object ArgsToStratoJson extends FunctionUnit1V[ScarecrowRuntime, String, AnyRef, String] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Global
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set.empty
  override def description: String =
    "Given the strato column op and some args, encode the args into strato JSON format"
  override def arguments: Seq[String] = Seq("strato column op", "strato column op args")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    stratoOp: String,
    stratoOpArgs: Seq[AnyRef]
  ): String = {
    stratoOp match {
      case "delete" | "deleteUnder" | "execute" | "invalidate" =>
        checkStratoOpNumArgs(stratoOp, stratoOpArgs, 1)
        Type.OBJECT.serializer.getStratoJson(stratoOpArgs.head)
      case "fetch" | "insert" | "put" | "scan" =>
        checkStratoOpNumArgs(stratoOp, stratoOpArgs, 2)
        Type.OBJECT.serializer.getStratoJson(stratoOpArgs.asJava)
      case _ => throw new IllegalArgumentException(s"Unknown strato op: $stratoOp")
    }
  }

  private def checkStratoOpNumArgs(op: String, args: Seq[AnyRef], expectedNumArgs: Int): Unit = {
    if (args.length != expectedNumArgs) {
      throw new IllegalArgumentException(
        s"Strato op $op expects $expectedNumArgs args, but received ${args.length}")
    }
  }
}

object GetTweetRtfLabels extends FunctionUnit1[ScarecrowRuntime, Long, Future[JSet[String]]] {

  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event

  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION

  override def downstreams: Set[DownstreamService] = Set(DownstreamServices.StratoSafetyLabels)

  override def description =
    "Given the tweet id, return all the safety labels assigned to a given tweet ID.\n"

  override def arguments: Seq[String] = Seq[String]("the tweet id")

  override def examples: Seq[String] = Seq[String]("GetTweetRtfLabels(12345)")

  override def evaluate(
    context: com.twitter.botmaker.Context[ScarecrowRuntime],
    tweetId: Long
  ): Future[JSet[String]] = context.getRuntime.fetcher10
    .getSafetyLabelMap(
      context.getRuntime.getMigrateContext(context),
      tweetId
    ).map {
      _.labels match {
        case null =>
          ImmutableSet.of()

        case map: JMap[JavaSafetyLabelType, JavaSafetyLabel] =>
          map.keySet.asScala.map(_.toString).asJava
      }
    }
}

object GetUserCredV2
    extends FunctionUnit1[ScarecrowRuntime, Long, Future[Option[UserCredV2Scores]]] {
  override def cacheLevel: CacheLevel = com.twitter.botmaker.ASTNode.CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] = Set(
    DownstreamServices.StratoGetUserCredV2
  )
  override def description: String =
    "Takes a user ID and returns their UserCred V2 score, mass, and snapshot timestamp."
  override def arguments: Seq[String] = Seq("User ID")
  override def evaluate(
    context: Context[ScarecrowRuntime],
    userId: Long
  ): Future[Option[UserCredV2Scores]] =
    context.getRuntime.fetcher20.stratoFetcher20.getUserCredV2(userId)
}

object GetUserHealthScores
    extends FunctionUnit2[ScarecrowRuntime, Long, Seq[String], Future[
      Map[String, Double]
    ]] {
  override def cacheLevel: CacheLevel = CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] =
    Set(DownstreamServices.StratoUserHealthScores)
  override def description: String =
    "Get User Health Scores from HSS given a user ID and a list of User Health Models"
  override def arguments: Seq[String] = Seq("User ID", "User Health Models")
  override def examples: Seq[String] =
    Seq("""GetUserHealthScores(123, List("SpammyUserScore", "UserNsfwAgathaScore"))""")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    userId: Long,
    models: Seq[String]
  ): Future[Map[String, Double]] = {
    val modelsFromStr = models.map { healthModel =>
      UserHealthModel.valueOf(healthModel) match {
        case Some(m) => m
        case _ =>
          throw new IllegalArgumentException(s"$healthModel is not a valid UserHealthModel")
      }
    }
    modelsFromStr.foreach(m =>
      context.getRuntime.statsReceiver
        .counter("hss", "signal_usage", "User", m.name, context.getTrackingId.toString).incr(1))
    context.getRuntime.fetcher20.getUserHealthScores(userId, modelsFromStr)
  }
}

object GetUserHealthSignals
    extends FunctionUnit2[ScarecrowRuntime, Long, Seq[String], Future[
      Map[String, SignalValue]
    ]] {
  override def cacheLevel: CacheLevel = CacheLevel.Event
  override def actionLevel: ActionLevel = ActionLevel.NO_ACTION
  override def downstreams: Set[DownstreamService] =
    Set(DownstreamServices.StratoUserHealthSignals)
  override def description: String =
    "Get User Health Signals from HSS given a user ID and a list of User Health Signals"
  override def arguments: Seq[String] = Seq("User ID", "User Health Signals")
  override def examples: Seq[String] =
    Seq("""GetUserHealthSignals(123, List("SpammyUserScoreDouble", "ActiveDaysLast7DaysLong"))""")

  override def evaluate(
    context: Context[ScarecrowRuntime],
    userId: Long,
    signals: Seq[String]
  ): Future[Map[String, SignalValue]] = {
    val signalsFromStr = signals.map { healthSignal =>
      UserHealthSignal.valueOf(healthSignal) match {
        case Some(m) => m
        case _ =>
          throw new IllegalArgumentException(s"$healthSignal is not a valid UserHealthSignal")
      }
    }
    signalsFromStr.foreach(m =>
      context.getRuntime.statsReceiver
        .counter("hss", "signal_usage", "User", m.name, context.getTrackingId.toString).incr(1))
    context.getRuntime.fetcher20.getUserHealthSignals(userId, signalsFromStr)
  }
}
