package com.twitter.simclusters_v2.common

object CosineSimilarityUtil {

  def sumOfSquares[T](v: Map[T, Double]): Double = {
    v.values.foldLeft(0.0) { (sum, value) => sum + value * value }
  }

  def sumOfSquaresArray(v: Array[Double]): Double = {
    v.foldLeft(0.0) { (sum, value) => sum + value * value }
  }

  def norm[T](v: Map[T, Double]): Double = {
    math.sqrt(sumOfSquares(v))
  }

  def norm(v: Array[Double]): Double = {
    math.sqrt(sumOfSquaresArray(v))
  }

  def normArray(v: Array[Double]): Double = {
    math.sqrt(sumOfSquaresArray(v))
  }

  def logNorm[T](v: Map[T, Double]): Double = {
    math.log(sumOfSquares(v) + 1)
  }

  def logNormArray(v: Array[Double]): Double = {
    math.log(sumOfSquaresArray(v) + 1)
  }

  def expScaledNorm[T](v: Map[T, Double], exponent: Double): Double = {
    math.pow(sumOfSquares(v), exponent)
  }

  def expScaledNormArray(v: Array[Double], exponent: Double): Double = {
    math.pow(sumOfSquaresArray(v), exponent)
  }

  def l1Norm[T](v: Map[T, Double]): Double = {
    v.values.foldLeft(0.0) { (sum, value) => sum + Math.abs(value) }
  }

  def l1NormArray(v: Array[Double]): Double = {
    v.foldLeft(0.0) { (sum, value) => sum + Math.abs(value) }
  }

  def applyNorm[T](v: Map[T, Double], norm: Double): Map[T, Double] = {
    if (norm == 0) v else v.mapValues(x => x / norm)
  }

  def applyNormArray(v: Array[Double], norm: Double): Array[Double] = {
    if (norm == 0) v else v.map(_ / norm)
  }

  def normalize[T](v: Map[T, Double], maybeNorm: Option[Double] = None): Map[T, Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.norm(v))
    applyNorm(v, norm)
  }

  def normalizeArray(
    v: Array[Double],
    maybeNorm: Option[Double] = None
  ): Array[Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.normArray(v))
    applyNormArray(v, norm)
  }

  def logNormalize[T](v: Map[T, Double], maybeNorm: Option[Double] = None): Map[T, Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.logNorm(v))
    applyNorm(v, norm)
  }

  def logNormalizeArray(
    v: Array[Double],
    maybeNorm: Option[Double] = None
  ): Array[Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.logNormArray(v))
    applyNormArray(v, norm)
  }

  def expScaledNormalize[T](
    v: Map[T, Double],
    exponent: Option[Double] = None,
    maybeNorm: Option[Double] = None
  ): Map[T, Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.expScaledNorm(v, exponent.getOrElse(0.3)))
    applyNorm(v, norm)
  }

  def expScaledNormalizeArray(
    v: Array[Double],
    exponent: Double,
    maybeNorm: Option[Double] = None
  ): Array[Double] = {
    val norm = maybeNorm.getOrElse(CosineSimilarityUtil.expScaledNormArray(v, exponent))
    applyNormArray(v, norm)
  }

  def dotProduct[T](v1: Map[T, Double], v2: Map[T, Double]): Double = {
    val comparer = v1.size - v2.size
    val smaller = if (comparer > 0) v2 else v1
    val bigger = if (comparer > 0) v1 else v2

    smaller.foldLeft(0.0) {
      case (sum, (id, value)) =>
        sum + bigger.getOrElse(id, 0.0) * value
    }
  }

  def dotProduct(v1: Array[Double], v2: Array[Double]): Double = {
    if (v1.length != v2.length) {
      0.0d
    } else {
      v1.indices.map { i =>
        v1(i) * v2(i)
      }.sum
    }
  }

  def cosineSimilarity(v1: Array[Double], v2: Array[Double]): Double = {
    if (v1.length != v2.length) {
      0.0d
    } else {
      val v1Norm = norm(v1)
      val v2Norm = norm(v2)

      if (v1Norm == 0 || v2Norm == 0) {
        0.0d
      } else {
        dotProduct(v1, v2) / (v1Norm * v2Norm)
      }
    }
  }

  def dotProductForSortedClusterAndScores(
    v1C: Array[Int],
    v1S: Array[Double],
    v2C: Array[Int],
    v2S: Array[Double]
  ): Double = {
    require(v1C.length == v1S.length)
    require(v2C.length == v2S.length)
    var i1 = 0
    var i2 = 0
    var product: Double = 0.0

    while (i1 < v1C.length && i2 < v2C.length) {
      if (v1C(i1) == v2C(i2)) {
        product += v1S(i1) * v2S(i2)
        i1 += 1
        i2 += 1
      } else if (v1C(i1) > v2C(i2)) {
        i2 += 1
      } else {
        i1 += 1
      }
    }
    product
  }
}
