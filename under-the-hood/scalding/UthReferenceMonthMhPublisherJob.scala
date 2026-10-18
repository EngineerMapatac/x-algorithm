package com.twitter.visibility.under_the_hood

import com.twitter.algebird.QTree
import com.twitter.algebird.QTreeSemigroup
import com.twitter.algebird.Semigroup
import com.twitter.scalding.Args
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.Execution
import com.twitter.scalding.RichDate
import com.twitter.scalding.TypedPipe
import com.twitter.common.util.Clock
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.DALWrite._
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.scalding_internal.job.TwitterExecutionApp
import com.twitter.scalding_internal.job.analytics_batch._
import com.twitter.scalding_internal.multiformat.format.keyval.KeyVal
import com.twitter.usersource.snapshot.flat.UsersourceFlatScalaDataset
import com.twitter.usersource.snapshot.flat.thriftscala.FlatUser
import com.twitter.visibility.under_the_hood.thriftscala._
import java.util.TimeZone

// Reference computations are experimental-only / unused now
class UthReferenceMonthMhPublisherApp {
  import UnderTheHoodCommon._
  import UthReferenceMonthMhPublisherApp._

  implicit val tz: TimeZone = DateOps.UTC

  def runOnDateRange(dateRange: DateRange, config: UthReferenceMhConfig): Execution[Unit] = {
    val asOfDay = yyyymmdd(dateRange.end.timestamp)
    val monthPublishEpoch = dateRange.end.timestamp
    val lookbackStart = dateRange.end - Days(config.dailyLookbackDays - 1)
    val dataFloor = RichDate(yyyymmddToMs(config.dailyDataFirstDay))
    val readStart = if (lookbackStart < dataFloor) dataFloor else lookbackStart
    val readRange = DateRange(readStart, dateRange.end)
    val basePath = config.outputPath

    val monthsInLookback: Set[Int] =
      Iterator
        .iterate(readStart.timestamp)(_ + DayMs)
        .takeWhile(_ <= dateRange.end.timestamp)
        .map(yyyymm)
        .toSet

    def readFromFloor[T](firstDay: Option[Int], read: DateRange => TypedPipe[T]): TypedPipe[T] =
      firstDay.fold(TypedPipe.empty: TypedPipe[T]) { day =>
        val floor = RichDate(yyyymmddToMs(day))
        val start = if (readStart < floor) floor else readStart
        if (start > dateRange.end) TypedPipe.empty else read(DateRange(start, dateRange.end))
      }

    val eligible = DAL.read(UthDailyEligiblePostsScalaDataset, readRange).toTypedPipe
    val postLabels = DAL.read(UthDailyPostLabelsScalaDataset, readRange).toTypedPipe ++
      readFromFloor(
        config.postTakedownDataFirstDay,
        DAL.read(UthDailyPostTakedownLabelsScalaDataset, _).toTypedPipe)
    val accountLabels = DAL.read(UthDailyAccountLabelsScalaDataset, readRange).toTypedPipe ++
      readFromFloor(
        config.accountTakedownDataFirstDay,
        DAL.read(UthDailyAccountTakedownLabelsScalaDataset, _).toTypedPipe)

    val followers =
      loadFollowers(dateRange, config.usersourceSnapshotMaxAgeDays, config.testUserIds)

    val lookbackStartYyyymmdd = yyyymmdd(readStart.timestamp)
    val referenceByMonth = buildReference(
      eligible,
      postLabels,
      accountLabels,
      followers,
      monthsInLookback,
      asOfDay,
      lookbackStartYyyymmdd,
      monthPublishEpoch,
      config.postObservationDays,
      config.testUserIds,
      config.reducers
    )

    val sharded = referenceByMonth.shard(config.writeShards)

    val mhWrite =
      sharded
        .map { case (month, value) => KeyVal(month, value) }
        .writeDALVersionedKeyValExecution(
          dataset = UthReferenceMonthAggregatesMhScalaDataset,
          pathLayout = D.Suffix(s"$basePath/mh/uth_reference_month_aggregates"),
          version = UseClockTime(Clock.SYSTEM_CLOCK)
        )

    val parquetWrite =
      sharded
        .map { case (_, value) => value }
        .writeDALSnapshotExecution(
          UthReferenceMonthAggregatesScalaDataset,
          D.Daily,
          D.Suffix(s"$basePath/uth_reference_month_aggregates"),
          D.Parquet,
          dateRange.end
        )

    Execution.zip(mhWrite, parquetWrite).unit
  }

  private def loadFollowers(
    dateRange: DateRange,
    usersourceSnapshotMaxAgeDays: Int,
    testUserIds: Set[Long]
  ): TypedPipe[(Long, Long)] = {
    val snapRange =
      DateRange(dateRange.end - Days(usersourceSnapshotMaxAgeDays), dateRange.end)
    DAL
      .readMostRecentSnapshot(UsersourceFlatScalaDataset, snapRange)
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .withColumns(Set("id", "followers", "deactivated", "suspended", "erased"))
      .toTypedPipe
      .flatMap { (user: FlatUser) =>
        val dropped =
          user.deactivated.contains(true) ||
            user.suspended.contains(true) ||
            user.erased.contains(true)
        user.id
          .filter(id => !dropped && inScope(testUserIds, id))
          .map(id => (id, user.followers.getOrElse(0L)))
      }
  }
}

object UthReferenceMonthMhPublisherApp {
  import UnderTheHoodCommon._

  private[under_the_hood] val FollowerDim = "followerClass"
  private[under_the_hood] val EligiblePostsDim = "eligiblePostsInMonth"

  private def followerEnum(name: String): UthFollowerClass = name match {
    case "LT_1K" => UthFollowerClass.Lt1k
    case "FROM_1K_TO_10K" => UthFollowerClass.From1kTo10k
    case "FROM_10K_TO_100K" => UthFollowerClass.From10kTo100k
    case "GTE_100K" => UthFollowerClass.Gte100k
    case _ => UthFollowerClass.All
  }

  private[under_the_hood] def eligiblePostVolumeValues(postCount: Long): Seq[String] = {
    val all = List("ALL")
    val gt10 = if (postCount > 10L) List("GT_10") else Nil
    val gt100 = if (postCount > 100L) List("GT_100") else Nil
    all ++ gt10 ++ gt100
  }

  private case class RateSketch(n: Long, sum: Double, tree: Option[QTree[Double]])

  private object RateSketch {
    private implicit val qTreeSemigroup: QTreeSemigroup[Double] = new QTreeSemigroup[Double](9)

    def one(rate: Double): RateSketch =
      RateSketch(1L, rate, if (rate > 0.0) Some(QTree(rate)) else None)

    implicit val semigroup: Semigroup[RateSketch] =
      Semigroup.from { (a, b) =>
        val tree = (a.tree, b.tree) match {
          case (Some(x), Some(y)) => Some(qTreeSemigroup.plus(x, y))
          case (x, y) => x.orElse(y)
        }
        RateSketch(a.n + b.n, a.sum + b.sum, tree)
      }

    def toStats(s: RateSketch): UthRateStats = {
      val nNonzero = s.tree.map(_.count).getOrElse(0L)
      val zeroFraction = if (s.n > 0L) (s.n - nNonzero).toDouble / s.n else 1.0
      def quantile(p: Double): Double =
        s.tree match {
          case Some(tree) if p > zeroFraction =>
            val tailP = math.min(1.0, (p - zeroFraction) / (1.0 - zeroFraction))
            val (lo, hi) = tree.quantileBounds(tailP)
            (lo + hi) / 2.0
          case _ => 0.0
        }
      UthRateStats(
        mean = Some(if (s.n > 0L) s.sum / s.n else 0.0),
        p10 = Some(quantile(0.10)),
        p25 = Some(quantile(0.25)),
        p50 = Some(quantile(0.50)),
        p75 = Some(quantile(0.75)),
        p90 = Some(quantile(0.90)),
        p99 = Some(quantile(0.99)),
        nUsers = Some(s.n)
      )
    }
  }

  private def cohortRow(
    dimension: String,
    value: String,
    userCount: Long,
    rateRows: List[(String, String, UthRateStats)]
  ): UthReferenceCohortStats = {
    val postRates = rateRows.collect {
      case ("post", label, stats) => UthLabeledRateStats(Some(label), Some(stats))
    }
    val accountRates = rateRows.collect {
      case ("acct", label, stats) => UthLabeledRateStats(Some(label), Some(stats))
    }
    if (dimension == FollowerDim)
      UthReferenceCohortStats(
        followerClass = Some(followerEnum(value)),
        nUsers = Some(userCount),
        postLabelAgg = Some(postRates),
        accountLabelAgg = Some(accountRates),
        // Experimental / not used: brand safety not implemented.
        brandSafetyAgg = Some(Nil)
      )
    else
      UthReferenceCohortStats(
        cohortDimension = Some(dimension),
        cohortValue = Some(value),
        nUsers = Some(userCount),
        postLabelAgg = Some(postRates),
        accountLabelAgg = Some(accountRates),
        // Experimental / not used: brand safety not implemented.
        brandSafetyAgg = Some(Nil)
      )
  }

  private[under_the_hood] def buildReference(
    eligible: TypedPipe[UthDailyEligiblePost],
    postLabels: TypedPipe[UthDailyPostLabel],
    accountLabels: TypedPipe[UthDailyAccountLabel],
    followers: TypedPipe[(Long, Long)],
    months: Set[Int],
    asOfDay: Int,
    lookbackStartYyyymmdd: Int,
    monthPublishEpoch: Long,
    postObservationDays: Int,
    testUserIds: Set[Long],
    reducers: Int
  ): TypedPipe[(Int, UthReferenceMonthAggregate)] = {

    val latestPostLabels = latestAsOfPostLabelRows(postLabels, reducers)
    def inWindow(day: Int): Boolean =
      day >= lookbackStartYyyymmdd && day <= asOfDay && months(monthBucket(day))

    val postsByUserMonth = sumByKey(
      eligible.flatMap { row =>
        for {
          userId <- row.userId if inScope(testUserIds, userId)
          day <- row.authoredYyyymmdd if inWindow(day)
          count <- row.count
        } yield ((userId, monthBucket(day)), count)
      },
      reducers
    )

    val postLabelCounts = {
      val perLabel = sumByKey(
        latestPostLabels.flatMap { row =>
          for {
            userId <- row.userId if inScope(testUserIds, userId)
            day <- row.authoredYyyymmdd if inWindow(day)
            label <- row.label.map(TakedownLabels.rollupLabel)
            carried <- row.carried
          } yield (((userId, monthBucket(day)), label), carried)
        },
        reducers
      )
      listByKey(
        perLabel.toTypedPipe.map {
          case ((userMonth, label), carried) => (userMonth, (label, carried))
        },
        reducers
      )
    }

    val accountLabelDays = {
      val perLabel = sumByKey(
        applyReducers(
          accountLabels.flatMap { row =>
            for {
              userId <- row.userId if inScope(testUserIds, userId)
              day <- row.dayYyyymmdd if inWindow(day)
              label <- row.label.map(TakedownLabels.rollupLabel)
            } yield (((userId, monthBucket(day)), label, day), 1)
          }.group,
          reducers
        ).sum.keys.map { case (userMonth, label, _) => ((userMonth, label), 1L) },
        reducers
      )
      listByKey(
        perLabel.toTypedPipe.map { case ((userMonth, label), days) => (userMonth, (label, days)) },
        reducers
      )
    }

    val followerByUser =
      if (reducers > 0) followers.group.withReducers(reducers) else followers.group

    val withFollowers = {
      val byUser = postsByUserMonth.toTypedPipe.map {
        case ((userId, month), postCount) => (userId, (month, postCount))
      }
      val grouped = if (reducers > 0) byUser.group.withReducers(reducers) else byUser.group
      val joined = grouped.join(followerByUser)
      (if (reducers > 0) joined.withReducers(reducers) else joined).toTypedPipe.map {
        case (userId, ((month, postCount), followerCount)) =>
          ((userId, month), (postCount, followerCount))
      }
    }

    val withPostLabels = {
      val grouped =
        if (reducers > 0) withFollowers.group.withReducers(reducers) else withFollowers.group
      val joined = grouped.leftJoin(postLabelCounts)
      (if (reducers > 0) joined.withReducers(reducers) else joined).toTypedPipe
    }

    val withAll = {
      val grouped =
        if (reducers > 0) withPostLabels.group.withReducers(reducers) else withPostLabels.group
      val joined = grouped.leftJoin(accountLabelDays)
      (if (reducers > 0) joined.withReducers(reducers) else joined).toTypedPipe
    }

    val coveredDaysByMonth: Map[Int, Int] =
      months.map { month =>
        val (first, last) = monthCoverageYyyymmdd(month, lookbackStartYyyymmdd, asOfDay)
        month -> inclusiveDayCount(first, last)
      }.toMap
    val coverageBoundsByMonth: Map[Int, (Int, Int)] =
      months.map { month =>
        month -> monthCoverageYyyymmdd(month, lookbackStartYyyymmdd, asOfDay)
      }.toMap

    val userMonthSamples = withAll.map {
      case ((_, month), (((postCount, followerCount), postLabelsOpt), accountOpt)) =>
        val denomPosts = math.max(1L, postCount)
        val denomAccountDays = math.max(1, coveredDaysByMonth.getOrElse(month, 1)).toDouble
        val postRates = postLabelsOpt
          .getOrElse(Nil).map {
            case (label, count) => label -> (count.toDouble / denomPosts)
          }.toMap
        val accountRates = accountOpt
          .getOrElse(Nil).map {
            case (label, days) => label -> (days.toDouble / denomAccountDays)
          }.toMap
        (month, postCount, followerClass(followerCount), postRates, accountRates)
    }

    val ratesByCohort = {
      def cohorts(
        month: Int,
        postCount: Long,
        followerBucket: String
      ): Seq[(Int, String, String)] = {
        val follower =
          Seq((month, FollowerDim, followerBucket), (month, FollowerDim, "ALL"))
        val volume = eligiblePostVolumeValues(postCount).map(v => (month, EligiblePostsDim, v))
        follower ++ volume
      }

      val usersByMonth = userMonthSamples.map {
        case (month, postCount, followerBucket, postRates, accountRates) =>
          (month, (postCount, followerBucket, postRates, accountRates))
      }
      val postLabelsByMonth = userMonthSamples.flatMap {
        case (month, _, _, postRates, _) =>
          postRates.keys.iterator.map(label => (month, label))
      }.distinct
      val accountLabelsByMonth = userMonthSamples.flatMap {
        case (month, _, _, _, accountRates) =>
          accountRates.keys.iterator.map(label => (month, label))
      }.distinct

      val postJoined: TypedPipe[
        (Int, ((Long, String, Map[String, Double], Map[String, Double]), String))
      ] =
        if (reducers > 0)
          usersByMonth.group.withReducers(reducers).join(postLabelsByMonth.group).toTypedPipe
        else usersByMonth.join(postLabelsByMonth)
      val postExpanded = postJoined.flatMap {
        case (month, ((postCount, followerBucket, postRates, _), label)) =>
          val rate = postRates.getOrElse(label, 0.0)
          cohorts(month, postCount, followerBucket).map {
            case (m, dim, value) => ((m, dim, value, "post", label), rate)
          }
      }

      val accountJoined: TypedPipe[
        (Int, ((Long, String, Map[String, Double], Map[String, Double]), String))
      ] =
        if (reducers > 0)
          usersByMonth.group.withReducers(reducers).join(accountLabelsByMonth.group).toTypedPipe
        else usersByMonth.join(accountLabelsByMonth)
      val accountExpanded = accountJoined.flatMap {
        case (month, ((postCount, followerBucket, _, accountRates), label)) =>
          val rate = accountRates.getOrElse(label, 0.0)
          cohorts(month, postCount, followerBucket).map {
            case (m, dim, value) => ((m, dim, value, "acct", label), rate)
          }
      }

      val expanded = postExpanded ++ accountExpanded
      val grouped = if (reducers > 0) expanded.group.withReducers(reducers) else expanded.group
      grouped
        .mapValues(RateSketch.one)
        .sum
        .toTypedPipe
        .map {
          case ((month, dim, value, family, label), sketch) =>
            ((month, dim, value), (family, label, RateSketch.toStats(sketch)))
        }
    }

    val cohortSizes = {
      val counts = userMonthSamples.flatMap {
        case (month, postCount, followerBucket, _, _) =>
          val follower =
            Seq(((month, FollowerDim, followerBucket), 1L), ((month, FollowerDim, "ALL"), 1L))
          val volume =
            eligiblePostVolumeValues(postCount).map(v => ((month, EligiblePostsDim, v), 1L))
          follower ++ volume
      }
      val grouped = if (reducers > 0) counts.group.withReducers(reducers) else counts.group
      grouped.sum
    }

    val ratesGrouped =
      if (reducers > 0) ratesByCohort.group.withReducers(reducers) else ratesByCohort.group

    val cohortsByMonth = ratesGrouped.toList
      .join(cohortSizes)
      .toTypedPipe
      .map {
        case ((month, dim, value), (rateRows, userCount)) =>
          (month, cohortRow(dim, value, userCount, rateRows))
      }

    val byMonth =
      if (reducers > 0) cohortsByMonth.group.withReducers(reducers).toList
      else cohortsByMonth.group.toList

    byMonth.toTypedPipe.map {
      case (month, cohorts) =>
        val (firstCov, lastCov) = coverageBoundsByMonth.getOrElse(
          month,
          monthCoverageYyyymmdd(month, lookbackStartYyyymmdd, asOfDay)
        )
        val coverage = UthDaysIncluded(
          snapshotEpoch = Some(monthPublishEpoch),
          firstCoveredDay = Some(firstCov),
          completeThroughDay = Some(lastCov),
          generatedAtMs = Some(monthPublishEpoch),
          postObservationDays = Some(postObservationDays),
          finalThroughDay = Some(finalThroughDay(lastCov, asOfDay, postObservationDays))
        )
        (month, UthReferenceMonthAggregate(Some(month), Some(coverage), Some(cohorts)))
    }
  }

  private def sumByKey[K: Ordering](pipe: TypedPipe[(K, Long)], reducers: Int) = {
    val g = if (reducers > 0) pipe.group.withReducers(reducers) else pipe.group
    g.sum
  }

  private def listByKey[K: Ordering, V](pipe: TypedPipe[(K, V)], reducers: Int) = {
    val g = if (reducers > 0) pipe.group.withReducers(reducers) else pipe.group
    g.toList
  }
}

case class UthReferenceMhConfig(
  testUserIds: Set[Long],
  reducers: Int,
  dailyLookbackDays: Int,
  postObservationDays: Int,
  dailyDataFirstDay: Int,
  postTakedownDataFirstDay: Option[Int],
  accountTakedownDataFirstDay: Option[Int],
  writeShards: Int,
  usersourceSnapshotMaxAgeDays: Int,
  outputPath: String)

object UthReferenceMhConfig {
  def fromArgs(args: Args): UthReferenceMhConfig =
    UthReferenceMhConfig(
      testUserIds = UnderTheHoodCommon.parseUserIds(args),
      reducers = args.int("reducers", 20000),
      dailyLookbackDays =
        UnderTheHoodCommon.preferredInt(args, "dailyLookbackDays", "lookbackDays", 90),
      postObservationDays =
        UnderTheHoodCommon.preferredInt(args, "postObservationDays", "observationDays", 7),
      dailyDataFirstDay = args.int("dailyDataFirstDay", 20260701),
      postTakedownDataFirstDay = args.optional("postTakedownDataFirstDay").map(_.toInt),
      accountTakedownDataFirstDay = args.optional("accountTakedownDataFirstDay").map(_.toInt),
      writeShards = {
        val n = args.int("writeShards", 50)
        require(n > 0, s"--writeShards must be > 0; got $n")
        n
      },
      usersourceSnapshotMaxAgeDays = args.int("usersourceSnapshotMaxAgeDays", 10),
      outputPath = args.optional("outputPath").getOrElse("/user/<hadoop-role>/under_the_hood")
    )
}

object UthReferenceMonthMhPublisherAdhoc
    extends UthReferenceMonthMhPublisherApp
    with TwitterExecutionApp {
  override def job: Execution[Unit] = Execution.withArgs { args =>
    runOnDateRange(UnderTheHoodDates.resolve(args), UthReferenceMhConfig.fromArgs(args))
  }
}

object UthReferenceMonthMhPublisherProd
    extends UthReferenceMonthMhPublisherApp
    with TwitterScheduledExecutionApp {
  implicit val dp: DateParser = DateParser.default
  override def scheduledJob: Execution[Unit] = {
    val execArgs = AnalyticsBatchExecutionArgs(
      batchDesc = BatchDescription("uth_reference_month_mh_publisher_prod"),
      firstTime = BatchFirstTime(RichDate("2026-07-31")),
      batchIncrement = BatchIncrement(Days(1))
    )
    Execution.withArgs { args =>
      AnalyticsBatchExecution(execArgs) { dateRange =>
        runOnDateRange(dateRange, UthReferenceMhConfig.fromArgs(args))
      }
    }
  }
}
