package com.twitter.simclusters_v2.scalding.common.matrix

import com.twitter.algebird.Semigroup
import com.twitter.bijection.Injection
import com.twitter.scalding.TypedPipe
import com.twitter.scalding.ValuePipe
import org.apache.avro.SchemaBuilder.ArrayBuilder
import scala.util.Random

case class SparseRowMatrix[R, C, V](
  pipe: TypedPipe[(R, Map[C, V])],
  isSkinnyMatrix: Boolean
)(
  implicit override val rowOrd: Ordering[R],
  override val colOrd: Ordering[C],
  override val numericV: Numeric[V],
  override val semigroupV: Semigroup[V],
  override val rowInj: Injection[R, Array[Byte]],
  override val colInj: Injection[C, Array[Byte]])
    extends TypedPipeMatrix[R, C, V] {

  override lazy val nnz: ValuePipe[Long] = {
    this
      .filter((_, _, v) => v != numericV.zero)
      .pipe
      .values
      .map(_.size.toLong)
      .sum
  }

  override def get(rowId: R, colId: C): ValuePipe[V] = {
    this.pipe
      .collect {
        case (i, values) if i == rowId =>
          values.collect {
            case (j, value) if j == colId => value
          }
      }
      .flatten
      .sum
  }

  override def getRow(rowId: R): TypedPipe[(C, V)] = {
    this.pipe.flatMap {
      case (i, values) if i == rowId =>
        values.toSeq
      case _ =>
        Nil
    }
  }

  override def getCol(colId: C): TypedPipe[(R, V)] = {
    this.pipe.flatMap {
      case (i, values) =>
        values.collect {
          case (j, value) if j == colId =>
            i -> value
        }
    }
  }

  override lazy val uniqueRowIds: TypedPipe[R] = {
    this.pipe.map(_._1).distinct
  }

  override lazy val uniqueColIds: TypedPipe[C] = {
    this.pipe.flatMapValues(_.keys).values.distinct
  }

  lazy val toSparseMatrix: SparseMatrix[R, C, V] = {
    SparseMatrix(this.pipe.flatMap {
      case (i, values) =>
        values.map { case (j, value) => (i, j, value) }
    })
  }

  lazy val toTypedPipe: TypedPipe[(R, Map[C, V])] = {
    this.pipe
  }

  def filter(fn: (R, C, V) => Boolean): SparseRowMatrix[R, C, V] = {
    SparseRowMatrix(
      this.pipe
        .map {
          case (i, values) =>
            i -> values.filter { case (j, v) => fn(i, j, v) }
        }
        .filter(_._2.nonEmpty),
      isSkinnyMatrix = this.isSkinnyMatrix
    )
  }

  def sampleRows(samplingRatio: Double): SparseRowMatrix[R, C, V] = {
    SparseRowMatrix(this.pipe.filter(_ => Random.nextDouble < samplingRatio), this.isSkinnyMatrix)
  }

  def filterRows(rows: TypedPipe[R]): SparseRowMatrix[R, C, V] = {
    SparseRowMatrix(this.pipe.join(rows.asKeys).mapValues(_._1), this.isSkinnyMatrix)
  }

  def filterCols(cols: TypedPipe[C]): SparseRowMatrix[R, C, V] = {
    this.toSparseMatrix.filterCols(cols).toSparseRowMatrix(this.isSkinnyMatrix)
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
  ): SparseRowMatrix[R1, C1, V1] = {
    SparseRowMatrix(
      this.pipe.flatMap {
        case (i, values) =>
          values
            .map {
              case (j, v) => fn(i, j, v)
            }
            .groupBy(_._1)
            .mapValues { _.map { case (_, j1, v1) => (j1, v1) }.toMap }
      },
      isSkinnyMatrix = this.isSkinnyMatrix
    )
  }

  lazy val rowL2Norms: TypedPipe[(R, Double)] = {
    this.pipe.map {
      case (row, values) =>
        row -> math.sqrt(
          values.values
            .map(a => numericV.toDouble(a) * numericV.toDouble(a))
            .sum)
    }
  }

  lazy val rowL2Normalize: SparseRowMatrix[R, C, Double] = {
    val result = this.pipe.flatMap {
      case (row, values) =>
        val norm =
          math.sqrt(
            values.values
              .map(v => numericV.toDouble(v) * numericV.toDouble(v))
              .sum)
        if (norm == 0.0) {
          None
        } else {
          Some(row -> values.mapValues(v => numericV.toDouble(v) / norm))
        }
    }

    SparseRowMatrix(result, isSkinnyMatrix = this.isSkinnyMatrix)
  }

  lazy val colL2Norms: TypedPipe[(C, Double)] = {
    this.pipe
      .flatMap {
        case (_, values) =>
          values.map {
            case (col, v) =>
              col -> numericV.toDouble(v) * numericV.toDouble(v)
          }
      }
      .sumByKey
      .mapValues(math.sqrt)
  }

  lazy val colL2Normalize: SparseRowMatrix[R, C, Double] = {
    val result = if (this.isSkinnyMatrix) {
      val colL2NormsValuePipe = this.colL2Norms.map {
        case (col, norm) => Map(col -> norm)
      }.sum

      this.pipe.flatMapWithValue(colL2NormsValuePipe) {
        case ((row, values), Some(colNorms)) =>
          Some(row -> values.flatMap {
            case (col, value) =>
              val colNorm = colNorms.getOrElse(col, 0.0)
              if (colNorm == 0.0) {
                None
              } else {
                Some(col -> numericV.toDouble(value) / colNorm)
              }
          })
        case _ =>
          None
      }
    } else {
      this.toSparseMatrix.transpose.rowAsKeys
        .join(this.colL2Norms)
        .collect {
          case (col, ((row, value), colNorm)) if colNorm > 0.0 =>
            row -> Map(col -> numericV.toDouble(value) / colNorm)
        }
        .sumByKey
        .toTypedPipe
    }

    SparseRowMatrix(result, isSkinnyMatrix = this.isSkinnyMatrix)
  }

  def sortWithTakePerRow(
    k: Int
  )(
    ordering: Ordering[(C, V)]
  ): TypedPipe[(R, Seq[(C, V)])] = {
    this.pipe.map {
      case (row, values) =>
        row -> values.toSeq.sorted(ordering).take(k)
    }
  }

  def sortWithTakePerCol(
    k: Int
  )(
    ordering: Ordering[(R, V)]
  ): TypedPipe[(C, Seq[(R, V)])] = {
    this.toSparseMatrix.sortWithTakePerCol(k)(ordering)
  }

  def forceToDisk(
    numShardsOpt: Option[Int] = None
  ): SparseRowMatrix[R, C, V] = {
    numShardsOpt
      .map { numShards =>
        SparseRowMatrix(this.pipe.shard(numShards), this.isSkinnyMatrix)
      }
      .getOrElse {
        SparseRowMatrix(this.pipe.forceToDisk, this.isSkinnyMatrix)
      }
  }

  def transposeAndMultiplySkinnySparseRowMatrix[C2](
    anotherSparseRowMatrix: SparseRowMatrix[R, C2, V],
    numReducersOpt: Option[Int] = None
  )(
    implicit ordering2: Ordering[C2],
    injection2: Injection[C2, Array[Byte]]
  ): SparseRowMatrix[C, C2, V] = {

    require(anotherSparseRowMatrix.isSkinnyMatrix)

    SparseRowMatrix(
      numReducersOpt
        .map { numReducers =>
          this.pipe
            .join(anotherSparseRowMatrix.pipe).withReducers(numReducers)
        }.getOrElse(this.pipe
          .join(anotherSparseRowMatrix.pipe))
        .flatMap {
          case (_, (row1, row2)) =>
            row1.map {
              case (col1, val1) =>
                col1 -> row2.mapValues(val2 => numericV.times(val1, val2))
            }
        }
        .sumByKey,
      isSkinnyMatrix = true
    )

  }

  def multiplyDenseRowMatrix(
    denseRowMatrix: DenseRowMatrix[C],
    numReducersOpt: Option[Int] = None
  ): DenseRowMatrix[R] = {
    this.toSparseMatrix.multiplyDenseRowMatrix(denseRowMatrix, numReducersOpt)
  }

  def toDenseRowMatrix(numCols: Int, colToIndexFunction: C => Int): DenseRowMatrix[R] = {
    DenseRowMatrix(this.pipe.map {
      case (row, colMap) =>
        val array = new Array[Double](numCols)
        colMap.foreach {
          case (col, value) =>
            val index = colToIndexFunction(col)
            assert(index < numCols && index >= 0, "The converted index is out of range!")
            array(index) = numericV.toDouble(value)
        }
        row -> array
    })
  }

}
