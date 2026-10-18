package com.twitter.simclusters_v2.scalding.common.matrix

import com.twitter.algebird.ArrayMonoid
import com.twitter.algebird.BloomFilterMonoid
import com.twitter.algebird.Monoid
import com.twitter.algebird.Semigroup
import com.twitter.algebird.Semigroup._
import com.twitter.bijection.Injection
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe

case class DenseRowMatrix[R](
  pipe: TypedPipe[(R, Array[Double])],
)(
  implicit val rowOrd: Ordering[R],
  val rowInj: Injection[R, Array[Byte]]) {

  lazy val semigroupArrayV: Semigroup[Array[Double]] = new ArrayMonoid[Double]()

  lazy val toSparseMatrix: SparseMatrix[R, Int, Double] = {
    this.toSparseRowMatrix.toSparseMatrix
  }

  lazy val toSparseRowMatrix: SparseRowMatrix[R, Int, Double] = {
    SparseRowMatrix(
      this.pipe.map {
        case (i, values) =>
          (i, values.zipWithIndex.collect { case (value, j) if value != 0.0 => (j, value) }.toMap)
      },
      isSkinnyMatrix = true)
  }

  lazy val toTypedPipe: TypedPipe[(R, Array[Double])] = {
    this.pipe
  }

  def filterRows(rows: TypedPipe[R]): DenseRowMatrix[R] = {
    DenseRowMatrix(this.pipe.join(rows.asKeys).mapValues(_._1))
  }

  lazy val rowL2Norms: TypedPipe[(R, Double)] = {
    this.pipe.map {
      case (row, values) =>
        row -> math.sqrt(values.map(a => a * a).sum)
    }
  }

  lazy val rowL2Normalize: DenseRowMatrix[R] = {

    DenseRowMatrix(this.pipe.map {
      case (row, values) =>
        val norm = math.sqrt(values.map(v => v * v).sum)
        if (norm == 0.0) {
          row -> values
        } else {
          row -> values.map(v => v / norm)
        }
    })
  }

}
