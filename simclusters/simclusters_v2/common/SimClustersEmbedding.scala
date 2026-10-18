package com.twitter.simclusters_v2.common

import com.twitter.simclusters_v2.thriftscala.SimClusterWithScore
import com.twitter.simclusters_v2.thriftscala.{SimClustersEmbedding => ThriftSimClustersEmbedding}
import scala.collection.mutable
import scala.language.implicitConversions
import scala.util.hashing.MurmurHash3.arrayHash
import scala.util.hashing.MurmurHash3.productHash
import scala.math._

sealed trait SimClustersEmbedding extends Equals {
  import SimClustersEmbedding._

  private[simclusters_v2] val clusterIds: Array[ClusterId]
  private[simclusters_v2] val scores: Array[Double]
  private[simclusters_v2] val sortedClusterIds: Array[ClusterId]
  private[simclusters_v2] val sortedScores: Array[Double]

  lazy val clusterIdSet: Set[ClusterId] = sortedClusterIds.toSet

  lazy val embedding: Seq[(ClusterId, Double)] =
    sortedClusterIds.zip(sortedScores).sortBy(-_._2).toSeq

  lazy val map: Map[ClusterId, Double] = sortedClusterIds.zip(sortedScores).toMap

  lazy val l1norm: Double = CosineSimilarityUtil.l1NormArray(sortedScores)

  lazy val l2norm: Double = CosineSimilarityUtil.normArray(sortedScores)

  lazy val logNorm: Double = CosineSimilarityUtil.logNormArray(sortedScores)

  lazy val expScaledNorm: Double =
    CosineSimilarityUtil.expScaledNormArray(sortedScores, DefaultExponent)

  lazy val normalizedSortedScores: Array[Double] =
    CosineSimilarityUtil.applyNormArray(sortedScores, l2norm)

  lazy val logNormalizedSortedScores: Array[Double] =
    CosineSimilarityUtil.applyNormArray(sortedScores, logNorm)

  lazy val expScaledNormalizedSortedScores: Array[Double] =
    CosineSimilarityUtil.applyNormArray(sortedScores, expScaledNorm)

  lazy val std: Double = {
    if (scores.isEmpty) {
      0.0
    } else {
      val sum = scores.sum
      val mean = sum / scores.length
      var variance: Double = 0.0
      for (i <- scores.indices) {
        val v = scores(i) - mean
        variance += (v * v)
      }
      math.sqrt(variance / scores.length)
    }
  }

  def get(clusterId: ClusterId): Option[Double] = {
    var i = 0
    while (i < sortedClusterIds.length) {
      val thisId = sortedClusterIds(i)
      if (clusterId == thisId) return Some(sortedScores(i))
      if (thisId > clusterId) return None
      i += 1
    }
    None
  }

  def getOrElse(clusterId: ClusterId, default: Double = 0.0): Double = {
    require(default >= 0.0)
    var i = 0
    while (i < sortedClusterIds.length) {
      val thisId = sortedClusterIds(i)
      if (clusterId == thisId) return sortedScores(i)
      if (thisId > clusterId) return default
      i += 1
    }
    default
  }

  def getClusterIds(): Array[ClusterId] = clusterIds

  def topClusterIds(size: Int): Seq[ClusterId] = clusterIds.take(size)

  def contains(clusterId: ClusterId): Boolean = clusterIdSet.contains(clusterId)

  def sum(another: SimClustersEmbedding): SimClustersEmbedding = {
    if (another.isEmpty) this
    else if (this.isEmpty) another
    else {
      var i1 = 0
      var i2 = 0
      val l = scala.collection.mutable.ArrayBuffer.empty[(Int, Double)]
      while (i1 < sortedClusterIds.length && i2 < another.sortedClusterIds.length) {
        if (sortedClusterIds(i1) == another.sortedClusterIds(i2)) {
          l += Tuple2(sortedClusterIds(i1), sortedScores(i1) + another.sortedScores(i2))
          i1 += 1
          i2 += 1
        } else if (sortedClusterIds(i1) > another.sortedClusterIds(i2)) {
          l += Tuple2(another.sortedClusterIds(i2), another.sortedScores(i2))
          i2 += 1
        } else {
          l += Tuple2(sortedClusterIds(i1), sortedScores(i1))
          i1 += 1
        }
      }
      if (i1 == sortedClusterIds.length && i2 != another.sortedClusterIds.length)
        l ++= another.sortedClusterIds.drop(i2).zip(another.sortedScores.drop(i2))
      else if (i1 != sortedClusterIds.length && i2 == another.sortedClusterIds.length)
        l ++= sortedClusterIds.drop(i1).zip(sortedScores.drop(i1))
      SimClustersEmbedding(l)
    }
  }

  def scalarMultiply(multiplier: Double): SimClustersEmbedding = {
    require(multiplier > 0.0, "SimClustersEmbedding.scalarMultiply requires multiplier > 0.0")
    DefaultSimClustersEmbedding(
      clusterIds,
      scores.map(_ * multiplier),
      sortedClusterIds,
      sortedScores.map(_ * multiplier)
    )
  }

  def scalarDivide(divisor: Double): SimClustersEmbedding = {
    require(divisor > 0.0, "SimClustersEmbedding.scalarDivide requires divisor > 0.0")
    DefaultSimClustersEmbedding(
      clusterIds,
      scores.map(_ / divisor),
      sortedClusterIds,
      sortedScores.map(_ / divisor)
    )
  }

  def dotProduct(another: SimClustersEmbedding): Double = {
    CosineSimilarityUtil.dotProductForSortedClusterAndScores(
      sortedClusterIds,
      sortedScores,
      another.sortedClusterIds,
      another.sortedScores)
  }

  def cosineSimilarity(another: SimClustersEmbedding): Double = {
    CosineSimilarityUtil.dotProductForSortedClusterAndScores(
      sortedClusterIds,
      normalizedSortedScores,
      another.sortedClusterIds,
      another.normalizedSortedScores)
  }

  def logNormCosineSimilarity(another: SimClustersEmbedding): Double = {
    CosineSimilarityUtil.dotProductForSortedClusterAndScores(
      sortedClusterIds,
      logNormalizedSortedScores,
      another.sortedClusterIds,
      another.logNormalizedSortedScores)
  }

  def expScaledCosineSimilarity(another: SimClustersEmbedding): Double = {
    CosineSimilarityUtil.dotProductForSortedClusterAndScores(
      sortedClusterIds,
      expScaledNormalizedSortedScores,
      another.sortedClusterIds,
      another.expScaledNormalizedSortedScores)
  }

  def isEmpty: Boolean = sortedClusterIds.isEmpty

  def jaccardSimilarity(another: SimClustersEmbedding): Double = {
    if (this.isEmpty || another.isEmpty) {
      0.0
    } else {
      val intersect = clusterIdSet.intersect(another.clusterIdSet).size
      val union = clusterIdSet.union(another.clusterIdSet).size
      intersect.toDouble / union
    }
  }

  def fuzzyJaccardSimilarity(another: SimClustersEmbedding): Double = {
    if (this.isEmpty || another.isEmpty) {
      0.0
    } else {
      val v1C = sortedClusterIds
      val v1S = sortedScores
      val v2C = another.sortedClusterIds
      val v2S = another.sortedScores

      require(v1C.length == v1S.length)
      require(v2C.length == v2S.length)

      var i1 = 0
      var i2 = 0
      var numerator = 0.0
      var denominator = 0.0

      while (i1 < v1C.length && i2 < v2C.length) {
        if (v1C(i1) == v2C(i2)) {
          numerator += min(v1S(i1), v2S(i2))
          denominator += max(v1S(i1), v2S(i2))
          i1 += 1
          i2 += 1
        } else if (v1C(i1) > v2C(i2)) {
          denominator += v2S(i2)
          i2 += 1
        } else {
          denominator += v1S(i1)
          i1 += 1
        }
      }

      while (i1 < v1C.length) {
        denominator += v1S(i1)
        i1 += 1
      }
      while (i2 < v2C.length) {
        denominator += v2S(i2)
        i2 += 1
      }

      numerator / denominator
    }
  }

  def euclideanDistance(another: SimClustersEmbedding): Double = {
    val unionClusters = clusterIdSet.union(another.clusterIdSet)
    val variance = unionClusters.foldLeft(0.0) {
      case (sum, clusterId) =>
        val distance = math.abs(this.getOrElse(clusterId) - another.getOrElse(clusterId))
        sum + distance * distance
    }
    math.sqrt(variance)
  }

  def manhattanDistance(another: SimClustersEmbedding): Double = {
    val unionClusters = clusterIdSet.union(another.clusterIdSet)
    unionClusters.foldLeft(0.0) {
      case (sum, clusterId) =>
        sum + math.abs(this.getOrElse(clusterId) - another.getOrElse(clusterId))
    }
  }

  def chiSquaredScore(globalEmbedding: SimClustersEmbedding): Double = {
    globalEmbedding.map.foldLeft(0.0) {
      case (sum, (clusterId, score)) => {
        sum + math.pow(map.getOrElse(clusterId, 0.0) - score, 2) / score
      }
    }
  }

  def overlappingClusters(another: SimClustersEmbedding): Int = {
    var i1 = 0
    var i2 = 0
    var count = 0

    while (i1 < sortedClusterIds.length && i2 < another.sortedClusterIds.length) {
      if (sortedClusterIds(i1) == another.sortedClusterIds(i2)) {
        count += 1
        i1 += 1
        i2 += 1
      } else if (sortedClusterIds(i1) > another.sortedClusterIds(i2)) {
        i2 += 1
      } else {
        i1 += 1
      }
    }
    count
  }

  def maxElementwiseProduct(another: SimClustersEmbedding): Double = {
    var i1 = 0
    var i2 = 0
    var maxProduct: Double = 0.0

    while (i1 < sortedClusterIds.length && i2 < another.sortedClusterIds.length) {
      if (sortedClusterIds(i1) == another.sortedClusterIds(i2)) {
        val product = sortedScores(i1) * another.sortedScores(i2)
        if (product > maxProduct) maxProduct = product
        i1 += 1
        i2 += 1
      } else if (sortedClusterIds(i1) > another.sortedClusterIds(i2)) {
        i2 += 1
      } else {
        i1 += 1
      }
    }
    maxProduct
  }

  def truncate(size: Int): SimClustersEmbedding = {
    if (clusterIds.length <= size) {
      this
    } else {
      val truncatedClusterIds = clusterIds.take(size)
      val truncatedScores = scores.take(size)
      val (sortedClusterIds, sortedScores) =
        truncatedClusterIds.zip(truncatedScores).sortBy(_._1).unzip

      DefaultSimClustersEmbedding(
        truncatedClusterIds,
        truncatedScores,
        sortedClusterIds,
        sortedScores)
    }
  }

  def toNormalized: SimClustersEmbedding = {
    if (l2norm == 0.0) {
      EmptyEmbedding
    } else {
      this.scalarDivide(l2norm)
    }
  }

  implicit def toThrift: ThriftSimClustersEmbedding = {
    ThriftSimClustersEmbedding(
      embedding.map {
        case (clusterId, score) =>
          SimClusterWithScore(clusterId, score)
      }
    )
  }

  def canEqual(a: Any): Boolean = a.isInstanceOf[SimClustersEmbedding]

  override def equals(that: Any): Boolean =
    that match {
      case that: SimClustersEmbedding =>
        that.canEqual(this) &&
          this.sortedClusterIds.sameElements(that.sortedClusterIds) &&
          this.sortedScores.sameElements(that.sortedScores)
      case _ => false
    }

  override lazy val hashCode: Int = {
    productHash((arrayHash(sortedClusterIds), arrayHash(sortedScores)))
  }
}

object SimClustersEmbedding {
  val EmptyEmbedding: SimClustersEmbedding =
    DefaultSimClustersEmbedding(Array.empty, Array.empty, Array.empty, Array.empty)

  val DefaultExponent: Double = 0.3

  implicit val order: Ordering[(ClusterId, Double)] =
    (a: (ClusterId, Double), b: (ClusterId, Double)) => {
      b._2 compare a._2 match {
        case 0 => a._1 compare b._1
        case c => c
      }
    }

  def apply(embedding: (ClusterId, Double)*): SimClustersEmbedding =
    buildDefaultSimClustersEmbedding(embedding)

  def apply(embedding: Iterable[(ClusterId, Double)]): SimClustersEmbedding =
    buildDefaultSimClustersEmbedding(embedding)

  def apply(embedding: Iterable[(ClusterId, Double)], size: Int): SimClustersEmbedding =
    buildDefaultSimClustersEmbedding(embedding, truncate = Some(size))

  implicit def apply(thriftEmbedding: ThriftSimClustersEmbedding): SimClustersEmbedding =
    buildDefaultSimClustersEmbedding(thriftEmbedding.embedding.map(_.toTuple))

  def apply(thriftEmbedding: ThriftSimClustersEmbedding, truncate: Int): SimClustersEmbedding =
    buildDefaultSimClustersEmbedding(
      thriftEmbedding.embedding.map(_.toTuple),
      truncate = Some(truncate))

  private def buildDefaultSimClustersEmbedding(
    embedding: Iterable[(ClusterId, Double)],
    truncate: Option[Int] = None
  ): SimClustersEmbedding = {
    val truncatedIdAndScores = {
      val idsAndScores = embedding.filter(_._2 > 0.0).toArray.sorted(order)
      truncate match {
        case Some(t) => idsAndScores.take(t)
        case _ => idsAndScores
      }
    }

    if (truncatedIdAndScores.isEmpty) {
      EmptyEmbedding
    } else {
      val (clusterIds, scores) = truncatedIdAndScores.unzip
      val (sortedClusterIds, sortedScores) = truncatedIdAndScores.sortBy(_._1).unzip
      DefaultSimClustersEmbedding(clusterIds, scores, sortedClusterIds, sortedScores)
    }
  }

  def sum(simClustersEmbeddings: Iterable[SimClustersEmbedding]): SimClustersEmbedding = {
    if (simClustersEmbeddings.isEmpty) {
      EmptyEmbedding
    } else {
      val sum = simClustersEmbeddings.foldLeft(mutable.Map[ClusterId, Double]()) {
        (sum, embedding) =>
          for (i <- embedding.sortedClusterIds.indices) {
            val clusterId = embedding.sortedClusterIds(i)
            sum.put(clusterId, embedding.sortedScores(i) + sum.getOrElse(clusterId, 0.0))
          }
          sum
      }
      SimClustersEmbedding(sum)
    }
  }

  def sum(
    simClustersEmbeddings: Iterable[SimClustersEmbedding],
    maxSize: Int
  ): SimClustersEmbedding = {
    sum(simClustersEmbeddings).truncate(maxSize)
  }

  def mean(simClustersEmbeddings: Iterable[SimClustersEmbedding]): SimClustersEmbedding = {
    if (simClustersEmbeddings.isEmpty) {
      EmptyEmbedding
    } else {
      sum(simClustersEmbeddings).scalarDivide(simClustersEmbeddings.size)
    }
  }

  def mean(
    simClustersEmbeddings: Iterable[SimClustersEmbedding],
    maxSize: Int
  ): SimClustersEmbedding = {
    mean(simClustersEmbeddings).truncate(maxSize)
  }
}

case class DefaultSimClustersEmbedding(
  override val clusterIds: Array[ClusterId],
  override val scores: Array[Double],
  override val sortedClusterIds: Array[ClusterId],
  override val sortedScores: Array[Double])
    extends SimClustersEmbedding {

  override def toString: String =
    s"DefaultSimClustersEmbedding(${clusterIds.zip(scores).mkString(",")})"
}

object DefaultSimClustersEmbedding {
  def apply(embedding: Seq[(ClusterId, Double)]): SimClustersEmbedding = SimClustersEmbedding(
    embedding)
}
