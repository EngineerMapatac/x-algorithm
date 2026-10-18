package com.twitter.simclusters_v2.scalding.common.matrix

import com.twitter.algebird.Aggregator
import com.twitter.algebird.Semigroup
import com.twitter.bijection.Injection
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe

abstract class TypedPipeMatrix[R, C, @specialized(Double, Int, Float, Long, Short) V] {
  implicit val semigroupV: Semigroup[V]
  implicit val numericV: Numeric[V]
  implicit val rowOrd: Ordering[R]
  implicit val colOrd: Ordering[C]
  implicit val rowInj: Injection[R, Array[Byte]]
  implicit val colInj: Injection[C, Array[Byte]]

  val nnz: ValuePipe[Long]

  val uniqueRowIds: TypedPipe[R]

  val uniqueColIds: TypedPipe[C]

  def getRow(rowId: R): TypedPipe[(C, V)]

  def getCol(colId: C): TypedPipe[(R, V)]

  def get(rowId: R, colId: C): ValuePipe[V]

  lazy val numUniqueRows: ValuePipe[Long] = {
    this.uniqueRowIds.aggregate(Aggregator.size)
  }

  lazy val numUniqueCols: ValuePipe[Long] = {
    this.uniqueColIds.aggregate(Aggregator.size)
  }
}
