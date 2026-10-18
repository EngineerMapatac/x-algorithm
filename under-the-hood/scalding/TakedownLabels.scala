package com.twitter.visibility.under_the_hood

import com.twitter.tseng.withholding.thriftscala.TakedownReason

object TakedownLabels {

  val WorldwideCountryCode = "xx"
  val WorldwideCopyrightCountryCode = "xy"

  sealed abstract class ReasonType(val name: String, val order: Int, val countryScoped: Boolean)
  object ReasonType {
    case object LegalRequest extends ReasonType("LegalRequest", 1, true)
    case object BystanderReport extends ReasonType("BystanderReport", 2, true)
    case object UnspecifiedReason extends ReasonType("UnspecifiedReason", 3, true)
    case object Dmca extends ReasonType("Dmca", 4, false)
    case object DmcaMedia extends ReasonType("is_dmca", 5, false)

    val all: Seq[ReasonType] =
      Seq(LegalRequest, BystanderReport, UnspecifiedReason, Dmca, DmcaMedia)
    private val byName: Map[String, ReasonType] = all.map(t => t.name -> t).toMap
    def fromName(name: String): Option[ReasonType] = byName.get(name)

    def fromAccountEventName(name: String): Option[ReasonType] =
      Option(name).map(_.trim.toUpperCase).collect {
        case "LEGAL_REQUEST" => LegalRequest
        case "BYSTANDER_REPORT" => BystanderReport
        case "UNSPECIFIED" => UnspecifiedReason
        case "DMCA" => Dmca
      }
  }

  case class Reason(reasonType: ReasonType, countryCode: Option[String]) {
    def sortKey: (Int, String) = (reasonType.order, countryCode.getOrElse(""))
  }

  val MediaDmca: Reason = Reason(ReasonType.DmcaMedia, None)
  val TweetDmca: Reason = Reason(ReasonType.Dmca, None)

  def normalizeCountryCode(cc: String): String = cc.trim.toLowerCase

  def fromWithholdingArm(arm: TakedownReason): Option[Reason] =
    arm match {
      case TakedownReason.LegalRequest(r) => Some(scoped(ReasonType.LegalRequest, r.countryCode))
      case TakedownReason.BystanderReport(r) =>
        Some(scoped(ReasonType.BystanderReport, r.countryCode))
      case TakedownReason.UnspecifiedReason(r) => Some(unspecified(r.countryCode))
      case TakedownReason.Dmca(_) => Some(TweetDmca)
      case _ => None
    }

  def unspecified(countryCode: String): Reason =
    Reason(ReasonType.UnspecifiedReason, Some(normalizeCountryCode(countryCode)))

  private def scoped(t: ReasonType, cc: String): Reason = Reason(t, Some(normalizeCountryCode(cc)))

  private val CountrySeparator = ", "
  private val LabelPattern = """^(\w+)(?:\((.*)\))?$""".r

  def label(reasonType: ReasonType, countryCodes: Iterable[String]): String = {
    val codes = countryCodes.map(normalizeCountryCode).filter(_.nonEmpty).toSeq.distinct.sorted
    if (!reasonType.countryScoped || codes.isEmpty) reasonType.name
    else s"${reasonType.name}(${codes.mkString(CountrySeparator)})"
  }

  def parseLabel(label: String): Option[(ReasonType, Seq[String])] =
    Option(label).flatMap {
      case LabelPattern(name, codes) =>
        ReasonType.fromName(name).map { t =>
          val ccs =
            if (codes == null || codes.isEmpty) Nil
            else codes.split(",").map(normalizeCountryCode).filter(_.nonEmpty).toSeq.sorted
          (t, ccs)
        }
      case _ => None
    }

  def splitForServing(label: String): (String, Option[Seq[String]]) =
    parseLabel(label) match {
      case Some((t, codes)) => (t.name, Some(codes))
      case None => (label, None)
    }

  def rollupLabel(label: String): String = parseLabel(label).fold(label)(_._1.name)

  def encodeReason(r: Reason): String =
    r.countryCode.fold(r.reasonType.name)(cc => s"${r.reasonType.name}:$cc")

  def decodeReason(encoded: String): Option[Reason] =
    if (encoded == null || encoded.isEmpty) None
    else
      encoded.split(":", 2) match {
        case Array(name, cc) if cc.nonEmpty =>
          ReasonType.fromName(name).map(t => Reason(t, Some(normalizeCountryCode(cc))))
        case Array(name) => ReasonType.fromName(name).map(t => Reason(t, None))
        case _ => None
      }

  def encodeReasons(reasons: Seq[Reason]): String = reasons.map(encodeReason).mkString(";")

  def decodeReasons(encoded: String): Seq[Reason] =
    if (encoded == null || encoded.isEmpty) Nil
    else encoded.split(";").toSeq.flatMap(decodeReason).distinct.sortBy(_.sortKey)
}
