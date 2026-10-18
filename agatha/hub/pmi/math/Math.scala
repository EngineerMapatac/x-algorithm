package com.twitter.hub.pmi.math

case class MathParams(
  pseudoCounts: Double,
  z: Double,
  positive: Boolean,
  pmiDiscount: Double,
  minAbsPMI: Double,
  epsilon: Double = 1e-20,
  standardErrorReg: Double = 1e-10)

private[pmi] object Math {

  def getSparsePMI(pmi: Double)(implicit mathParams: MathParams): Option[Double] = {
    val discountedPMI = pmi - mathParams.pmiDiscount
    if ((scala.math.abs(discountedPMI) < mathParams.minAbsPMI)
      || (discountedPMI < 0 && mathParams.positive))
      None
    else
      Some(discountedPMI)
  }

  def bayesianAverage(
    empiricalSum: Double,
    totalCount: Double,
    priorMean: Double,
    pseudocounts: Double
  ): Double = {

    val sumSmoothed = empiricalSum + (priorMean * pseudocounts)
    val totalCountSmoothed = totalCount + pseudocounts

    sumSmoothed / totalCountSmoothed

  }

  def confidenceIntervalCorrection(
    p: Double,
    upper: Double,
    lower: Double,
    expectation: Double
  ): Double = {
    if (p > expectation) {
      math.max(lower, expectation)
    } else {
      math.min(upper, expectation)
    }
  }

  def computePMIExpWithVariance(
    sumVar1TimesVar2: Double,
    sumVar1TimesVar2Squared: Double,
    sumVar1: Double,
    sumVar2: Double,
    totalCount: Double
  )(
    implicit mathParams: MathParams
  ): Double = {

    val meanVar1 = sumVar1 / totalCount
    val meanVar2 = sumVar2 / totalCount

    val expectedMeanVar1TimesVar2 = meanVar1 * meanVar2

    val smoothedMeanVar1TimesVar2 = math.max(
      bayesianAverage(
        sumVar1TimesVar2,
        totalCount,
        expectedMeanVar1TimesVar2,
        mathParams.pseudoCounts),
      mathParams.epsilon
    )

    val meanVar1TimesVar2 = sumVar1TimesVar2 / totalCount
    val meanVar1TimesVar2Squared =
      math.max(sumVar1TimesVar2Squared / totalCount, mathParams.standardErrorReg)
    val varianceVar1TimesVar2 = meanVar1TimesVar2Squared - (meanVar1TimesVar2 * meanVar1TimesVar2)

    val standardError = math.sqrt(varianceVar1TimesVar2 / totalCount)

    val correctionTerm = mathParams.z * standardError
    val (lower, upper) =
      (smoothedMeanVar1TimesVar2 - correctionTerm, smoothedMeanVar1TimesVar2 + correctionTerm)

    val correctedProb = confidenceIntervalCorrection(
      smoothedMeanVar1TimesVar2,
      upper,
      lower,
      expectedMeanVar1TimesVar2
    )

    math.log(correctedProb) - math.log(expectedMeanVar1TimesVar2)
  }

  def computePMI(
    sumVar1TimesVar2: Double,
    sumVar1: Double,
    sumVar2: Double,
    totalCount: Double
  ) = {
    math.log((sumVar1TimesVar2 / totalCount) / (sumVar1 / totalCount * sumVar2 / totalCount))
  }
}
