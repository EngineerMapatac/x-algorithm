package com.twitter.simclusters_v2.scalding.common.matrix

import com.twitter.algebird.Semigroup
import com.twitter.bijection.Injection
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe

case class SparseMatrix[R, C, V](
  pipe: TypedPipe[(R, C, V)]
)(
  implicit override val rowOrd: Ordering[R],
  override val colOrd: Ordering[C],
  override val numericV: Numeric[V],
  override val semigroupV: Semigroup[V],
  override val rowInj: Injection[R, Array[Byte]],
  override val colInj: Injection[C, Array[Byte]])
    extends TypedPipeMatrix[R, C, V] {

  override lazy val nnz: ValuePipe[Long] = {
    this.filter((_, _, v) => v != numericV.zero).pipe.map(_ => 1L).sum
  }

  lazy val rowNnz: TypedPipe[(R, Long)] = {
    this.pipe.collect {
      case (row, _, v) if v != numericV.zero =>
        row -> 1L
    }.sumByKey
  }

  lazy val colNnz: TypedPipe[(C, Long)] = {
    this.transpose.rowNnz
  }

  override lazy val uniqueRowIds: TypedPipe[R] = {
    this.pipe.map(t => t._1).distinct
  }

  override lazy val uniqueColIds: TypedPipe[C] = {
    this.pipe.map(t => t._2).distinct
  }

  override def getRow(rowId: R): TypedPipe[(C, V)] = {
    this.pipe.collect {
      case (i, j, value) if i == rowId =>
        j -> value
    }
  }

  override def getCol(colId: C): TypedPipe[(R, V)] = {
    this.pipe.collect {
      case (i, j, value) if j == colId =>
        i -> value
    }
  }

  override def get(rowId: R, colId: C): ValuePipe[V] = {
    this.pipe.collect {
      case (i, j, value) if i == rowId && j == colId =>
        value
    }.sum
  }

  def filter(fn: (R, C, V) => Boolean): SparseMatrix[R, C, V] = {
    SparseMatrix(this.pipe.filter {
      case (row, col, value) => fn(row, col, value)
    })
  }

  def filterRows(rows: TypedPipe[R]): SparseMatrix[R, C, V] = {
    SparseMatrix(this.rowAsKeys.join(rows.asKeys).map {
      case (row, ((col, value), _)) => (row, col, value)
    })
  }

  def filterCols(cols: TypedPipe[C]): SparseMatrix[R, C, V] = {
    this.transpose.filterRows(cols).transpose
  }

  def tripleApply[R1, C1, V1](
    fn: (R, C, V) => (R1, C1, V1)
  )(
    implicit rowOrd1: Ordering[R1],
    colOrd1: Ordering[C1],
    numericV1: Numeric[V1],
    semigroupV1: Semigroup[V1],
    rowInj: Injection[R1, Array[Byte]],
    colInj: Injection[C1, Array[Byte]]
  ): SparseMatrix[R1, C1, V1] = {
    SparseMatrix(this.pipe.map {
      case (row, col, value) => fn(row, col, value)
    })
  }

  lazy val rowL1Norms: TypedPipe[(R, Double)] = {
    this.pipe.map {
      case (row, _, value) =>
        row -> numericV.toDouble(value).abs
    }.sumByKey
  }

  lazy val rowL2Norms: TypedPipe[(R, Double)] = {
    this.pipe
      .map {
        case (row, _, value) =>
          row -> numericV.toDouble(value) * numericV.toDouble(value)
      }
      .sumByKey
      .mapValues(math.sqrt)
  }

  lazy val rowL2Normalize: SparseMatrix[R, C, Double] = {
    val result = this.rowAsKeys
      .join(this.rowL2Norms)
      .collect {
        case (row, ((col, value), l2norm)) if l2norm > 0.0 =>
          (row, col, numericV.toDouble(value) / l2norm)
      }

    SparseMatrix(result)
  }

  lazy val colL2Norms: TypedPipe[(C, Double)] = {
    this.transpose.rowL2Norms
  }

  lazy val colL2Normalize: SparseMatrix[R, C, Double] = {
    this.transpose.rowL2Normalize.transpose
  }

  def sortWithTakePerRow(k: Int)(ordering: Ordering[(C, V)]): TypedPipe[(R, Seq[(C, V)])] = {
    this.rowAsKeys.group.sortedTake(k)(ordering)
  }

  def sortWithTakePerCol(k: Int)(ordering: Ordering[(R, V)]): TypedPipe[(C, Seq[(R, V)])] = {
    this.transpose.sortWithTakePerRow(k)(ordering)
  }

  def multiplySparseMatrix[C2](
    sparseMatrix: SparseMatrix[C, C2, V],
    numReducersOpt: Option[Int] = None
  )(
    implicit ordering2: Ordering[C2],
    injection2: Injection[C2, Array[Byte]]
  ): SparseMatrix[R, C2, V] = {
    implicit val colInjectionFunction: C => Array[Byte] = colInj.toFunction

    val result =
      this.transpose.rowAsKeys
        .sketch(numReducersOpt.getOrElse(1000))
        .join(sparseMatrix.rowAsKeys)
        .map {
          case (_, ((row1, value1), (col2, value2))) =>
            (row1, col2) -> numericV.times(value1, value2)
        }
        .sumByKey
        .map {
          case ((row, col), value) =>
            (row, col, value)
        }

    SparseMatrix(result)
  }

  def multiplySkinnySparseRowMatrix[C2](
    skinnyMatrix: SparseRowMatrix[C, C2, V],
    numReducersOpt: Option[Int] = None
  )(
    implicit ordering2: Ordering[C2],
    injection2: Injection[C2, Array[Byte]]
  ): SparseRowMatrix[R, C2, V] = {

    assert(
      skinnyMatrix.isSkinnyMatrix,
      "this function only works for skinny sparse row matrix, otherwise you will get out-of-memory problem")

    implicit val colInjectionFunction: C => Array[Byte] = colInj.toFunction

    val result =
      this.transpose.rowAsKeys
        .sketch(numReducersOpt.getOrElse(1000))
        .join(skinnyMatrix.pipe)
        .map {
          case (_, ((row1, value1), colMap)) =>
            row1 -> colMap.mapValues(v => numericV.times(value1, v))
        }
        .sumByKey

    SparseRowMatrix(result, skinnyMatrix.isSkinnyMatrix)
  }

  def multiplyDenseRowMatrix(
    denseRowMatrix: DenseRowMatrix[C],
    numReducersOpt: Option[Int] = None
  ): DenseRowMatrix[R] = {

    implicit val colInjectionFunction: C => Array[Byte] = colInj.toFunction
    implicit val arrayVSemiGroup: Semigroup[Array[Double]] = denseRowMatrix.semigroupArrayV

    val result =
      this.transpose.rowAsKeys
        .sketch(numReducersOpt.getOrElse(1000))
        .join(denseRowMatrix.pipe)
        .map {
          case (_, ((row1, value1), array)) =>
            row1 -> array.map(v => numericV.toDouble(value1) * v)
        }
        .sumByKey

    DenseRowMatrix(result)
  }

  lazy val transpose: SparseMatrix[C, R, V] = {
    SparseMatrix(
      this.pipe
        .map {
          case (row, col, value) =>
            (col, row, value)
        })
  }

  lazy val rowAsKeys: TypedPipe[(R, (C, V))] = {
    this.pipe
      .map {
        case (row, col, value) =>
          (row, (col, value))
      }
  }

  lazy val toTypedPipe: TypedPipe[(R, C, V)] = {
    this.pipe
  }

  lazy val forceToDisk: SparseMatrix[R, C, V] = {
    SparseMatrix(this.pipe.forceToDisk)
  }

  def toSparseRowMatrix(isSkinnyMatrix: Boolean = false): SparseRowMatrix[R, C, V] = {
    SparseRowMatrix(
      this.pipe.map {
        case (i, j, v) =>
          i -> Map(j -> v)
      }.sumByKey,
      isSkinnyMatrix)
  }

  def toDenseRowMatrix(numCols: Int, colToIndexFunction: C => Int): DenseRowMatrix[R] = {
    this.toSparseRowMatrix(isSkinnyMatrix = true).toDenseRowMatrix(numCols, colToIndexFunction)
  }

  private[this] def filterIter(
    columnValueIterator: Iterator[(C, V)],
    threshold: V,
    ifMin: Boolean
  ): Iterator[(C, V)] = {
    var sum: V = numericV.zero
    var it: Iterator[(C, V)] = Iterator.empty
    var exceeded = false
    while (columnValueIterator.hasNext && !exceeded) {
      val (c, v) = columnValueIterator.next
      val nextSum = semigroupV.plus(sum, v)
      val cmp = numericV.compare(nextSum, threshold)
      if ((ifMin && cmp < 0) || (!ifMin && cmp <= 0)) {
        it = it ++ Iterator((c, v))
        sum = nextSum
      } else {
        it = it ++ Iterator((c, v))
        exceeded = true
      }
    }
    (ifMin, exceeded) match {
      case (true, true) => it ++ columnValueIterator
      case (true, false) => Iterator.empty
      case (false, true) => Iterator.empty
      case (false, false) => it ++ columnValueIterator
    }
  }

  def filterRowsByMinSum(minSum: V): SparseMatrix[R, C, V] = {
    val filteredPipe = this.rowAsKeys.group
      .mapValueStream(filterIter(_, threshold = minSum, ifMin = true)).map {
        case (r, (c, v)) =>
          (r, c, v)
      }
    SparseMatrix(filteredPipe)
  }

  def filterRowsByMaxSum(maxSum: V): SparseMatrix[R, C, V] = {
    val filteredPipe = this.rowAsKeys.group
      .mapValueStream(filterIter(_, threshold = maxSum, ifMin = false)).map {
        case (r, (c, v)) =>
          (r, c, v)
      }
    SparseMatrix(filteredPipe)
  }
}
