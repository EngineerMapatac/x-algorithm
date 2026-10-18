package com.twitter.visibility.under_the_hood

import com.twitter.scalding.Args
import com.twitter.scalding.DateParser
import com.twitter.scalding.DateRange
import com.twitter.scalding.RichDate
import java.util.Calendar
import java.util.TimeZone

object UnderTheHoodDates {

  def resolve(args: Args): DateRange = {
    implicit val tz: TimeZone = TimeZone.getTimeZone("UTC")
    implicit val dp: DateParser = DateParser.default
    val dates = args.list("date")
    if (dates.nonEmpty) DateRange.parse(dates)
    else
      args
        .optional("month")
        .map(monthRange)
        .getOrElse(
          throw new IllegalArgumentException(
            "Adhoc runs require a window: pass --date START END or --month YYYY-MM " +
              "(production windows come from statebird via the *Prod scheduled app)."))
  }

  def monthRange(ym: String): DateRange = {
    require(ym.matches("""\d{4}-\d{2}"""), s"--month must be YYYY-MM; got '$ym'")
    val Array(year, month) = ym.split("-").map(_.toInt)
    require(month >= 1 && month <= 12, s"--month has an invalid month: '$ym'")
    val cal = utcCal()
    cal.setLenient(false)
    cal.clear()
    cal.set(year, month - 1, 1, 0, 0, 0)
    val start = cal.getTimeInMillis
    cal.add(Calendar.MONTH, 1)
    DateRange(RichDate(start), RichDate(cal.getTimeInMillis - 1L))
  }

  private def utcCal(): Calendar = Calendar.getInstance(TimeZone.getTimeZone("UTC"))
}
