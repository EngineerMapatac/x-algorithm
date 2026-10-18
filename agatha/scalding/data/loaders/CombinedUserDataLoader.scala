package com.twitter.agatha.scalding.data.loaders

import com.twitter.agatha.scalding.data.DataLoader
import com.twitter.common.text.pipeline.TwitterLanguageIdentifier
import com.twitter.common_internal.text.version.PenguinVersion
import com.twitter.conversions.DurationOps._
import com.twitter.gizmoduck.thriftscala.FrictionlessFollowerState
import com.twitter.gizmoduck.thriftscala.Safety
import com.twitter.gizmoduck.thriftscala.User
import com.twitter.hub.agatha.thriftscala.Feature
import com.twitter.hub.agatha.thriftscala.FeatureClass
import com.twitter.hub.agatha.thriftscala.FeatureIdentifier
import com.twitter.hub.agatha.thriftscala.FeatureSource
import com.twitter.scalding.DateOps
import com.twitter.scalding.DateRange
import com.twitter.scalding.Days
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.serialization.RequiredBinaryComparators._
import com.twitter.scalding_internal.dalv2.DAL
import com.twitter.scalding_internal.dalv2.remote_access.ExplicitLocation
import com.twitter.scalding_internal.dalv2.remote_access.ProcAtla
import com.twitter.search.common.util.text.TokenizerHelper
import com.twitter.twadoop.user.gen.thriftscala.CombinedUser
import com.twitter.twadoop.user.gen.thriftscala.UserExtended
import com.twitter.usersource.snapshot.combined.UsersourceScalaDataset

object CombinedUserDataLoader extends DataLoader {
  import collection.JavaConverters._
  import com.twitter.agatha.scalding.data.DataLoaderUtil._

  private def logInt(value: Double, scale: Double): Long = {
    math.ceil(math.log(1 + value * scale)).toLong
  }

  private val languageIdentifier = new TwitterLanguageIdentifier.Builder().buildForHighPrecision()

  private def getAccountCreationTimeFeatures(user: User) = {
    val days = user.createdAtMsec.millis.inDays
    val createdWeek = days / 7L
    val createdMonth = days / 30L
    val createdYear = days / 365L

    Set(
      ("create_week", createdWeek.toString),
      ("create_month", createdMonth.toString),
      ("create_year", createdYear.toString)
    )
  }

  private def getSignupIpFeatures(user: User) = {
    import com.twitter.geoduck.netutils.IPV4AddressUtils._

    val createdVia = user.account.map(_.createdVia).getOrElse("")
    val creationIpOpt = user.account.flatMap(_.creationIp)

    val ips = for {
      ip <- creationIpOpt
      ipInt <- ipStringToInt(ip)
      ipStr = ipIntToString(ipInt)
      ip2 = ipIntToString(ipInt & subnetMask(16))
      ip3 = ipIntToString(ipInt & subnetMask(24))
    } yield (ipStr, ip2, ip3)

    val (creationIpAll, creationIp2, creationIp3) = ips.getOrElse(("", "", ""))

    Set(
      ("created_via", createdVia),
      ("creation_ip", creationIpAll),
      ("creation_ip2", creationIp2),
      ("creation_ip3", creationIp3)
    )
  }

  private def getCountFeaturesFromUserExtended(userExtended: UserExtended) = {
    val followerCount = userExtended.followers.getOrElse("0").toLong
    val followingCount = userExtended.followings.getOrElse("0").toLong
    val activeFollowerCount = userExtended.activeFollowers.getOrElse("0").toLong

    Set(
      ("follower_count_log", logInt(followerCount, 1.0).toString),
      ("following_count_log", logInt(followingCount, 1.0).toString),
      ("active_follower_count_log", logInt(activeFollowerCount, 1.0).toString)
    )
  }

  private def getSignupGeoFeaturesFomUserExtended(userExtended: UserExtended) = {
    val signupCountry = userExtended.signupCountryId.getOrElse("")
    val signupRegion = userExtended.signupRegionDesc.getOrElse("")

    Set(
      ("signup_country", signupCountry),
      ("signup_region", signupRegion)
    )
  }

  private def getClientFeatures(user: User) = {
    val pushDevices = user.devices
      .map(_.messagingDevices.flatMap(_.carrierId))
      .toSeq.flatten
      .toSet

    val pushClients = user.devices
      .map(_.appPushDevices.flatMap(_.clientApplicationId))
      .toSeq.flatten
      .toSet

    pushDevices
      .map { device => ("push_device", device) } ++
      pushClients
        .map { client => ("push_client", client.toString) }
  }

  private def getUserProfileFeatures(user: User) = {
    val userProfileTokens = user.profile
      .map { profile =>
        try {
          val profileLang = languageIdentifier.identify(profile.description).getLocale
          TokenizerHelper
            .tokenizeTweet(profile.description, profileLang, PenguinVersion.PENGUIN_6)
            .tokens
            .asScala
            .distinct
            .toList
        } catch {
          case _: IllegalArgumentException =>
            Nil
        }
      }
      .getOrElse(Nil)

    userProfileTokens.map { token => ("profile_token", token) }.toSet
  }

  private def getSafetyFeatures(safety: Safety) = {
    Set(
      ("signup_category", safety.signupCategory.value.toString),
      ("is_verified", safety.verified.toString)
    )
  }

  private[loaders] def processCombinedUser(cu: CombinedUser) = {
    val features = for {
      user <- cu.user
      userExtended <- cu.userExtended
      safety <- user.safety
      spammer = {
        safety.restricted ||
        safety.frictionlessFollowerState == FrictionlessFollowerState.Frictionless ||
        safety.deactivated
      }
      if !spammer
      userId = user.id
      if notTestAccount(userId)
    } yield {
      val emailState = userExtended.emailState.getOrElse("")

      val features = List(("email_state", emailState)) ++
        getAccountCreationTimeFeatures(user) ++
        getSignupIpFeatures(user) ++
        getCountFeaturesFromUserExtended(userExtended) ++
        getSignupGeoFeaturesFomUserExtended(userExtended) ++
        getClientFeatures(user) ++
        getUserProfileFeatures(user) ++
        getSafetyFeatures(safety)

      features
        .map {
          case (featureType, value) =>
            (
              userId,
              FeatureIdentifier(
                FeatureClass(FeatureSource.CombinedUser, ""),
                featureType + ":" + value))
        }
    }

    features.toList.flatten
  }

  override def read(dateRange: DateRange): TypedPipe[(Long, Feature)] = {
    DAL
      .readMostRecentSnapshot(
        UsersourceScalaDataset,
        DateRange(dateRange.start - Days(7)(DateOps.UTC), dateRange.end)
      )
      .withRemoteReadPolicy(ExplicitLocation(ProcAtla))
      .toTypedPipe
      .flatMap(processCombinedUser)
      .map { case (uId, featureIdentifier) => (uId, Feature(featureIdentifier, 1.0)) }
  }

}
