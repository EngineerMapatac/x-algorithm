package com.twitter.hub.pmi

import com.twitter.hub.pmi.ModelCommon.getDataWithSumsHash
import com.twitter.hub.pmi.ModelCommon.getDataWithSumsSketch
import com.twitter.scalding.TypedPipe

object PMITypes {
  abstract class PMIDataContainer[T, U, V](
    pipe1: TypedPipe[(T, U, Double)],
    pipe2: TypedPipe[(T, V, Double)]) {
    val data1 = pipe1
    val data2 = pipe2

    private[pmi] def joinFeatureSumsToPipe1(
      regularizedPipe: TypedPipe[(T, U, Double)],
      featureSums: TypedPipe[(U, Double)]
    )(
      implicit ordering: Ordering[U],
      serializer: U => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[U])]
    private[pmi] def joinFeatureSumsToPipe2(
      regularizedPipe: TypedPipe[(T, V, Double)],
      featureSums: TypedPipe[(V, Double)]
    )(
      implicit ordering: Ordering[V],
      serializer: V => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[V])]
  }
  case class BigToBigPMIContainer[T, U, V](
    bigPipe1: TypedPipe[(T, U, Double)],
    bigPipe2: TypedPipe[(T, V, Double)])
      extends PMIDataContainer[T, U, V](bigPipe1, bigPipe2) {
    override private[pmi] def joinFeatureSumsToPipe1(
      regularizedPipe: TypedPipe[(T, U, Double)],
      featureSums: TypedPipe[(U, Double)]
    )(
      implicit ordering: Ordering[U],
      serializer: U => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[U])] = {
      getDataWithSumsSketch(regularizedPipe, featureSums)
    }

    override private[pmi] def joinFeatureSumsToPipe2(
      regularizedPipe: TypedPipe[(T, V, Double)],
      featureSums: TypedPipe[(V, Double)]
    )(
      implicit ordering: Ordering[V],
      serializer: V => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[V])] = {
      getDataWithSumsSketch(regularizedPipe, featureSums)
    }
  }
  case class BigToSmallPMIContainer[T, U, V](
    bigPipe: TypedPipe[(T, U, Double)],
    smallPipe: TypedPipe[(T, V, Double)])
      extends PMIDataContainer[T, U, V](bigPipe, smallPipe) {
    override private[pmi] def joinFeatureSumsToPipe1(
      regularizedPipe: TypedPipe[(T, U, Double)],
      featureSums: TypedPipe[(U, Double)]
    )(
      implicit ordering: Ordering[U],
      serializer: U => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[U])] = {
      getDataWithSumsSketch(regularizedPipe, featureSums)
    }

    override private[pmi] def joinFeatureSumsToPipe2(
      regularizedPipe: TypedPipe[(T, V, Double)],
      featureSums: TypedPipe[(V, Double)]
    )(
      implicit ordering: Ordering[V],
      serializer: V => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[V])] = {
      getDataWithSumsHash(regularizedPipe, featureSums)
    }
  }

  case class SmallToSmallPMIContainer[T, U, V](
    smallPipe1: TypedPipe[(T, U, Double)],
    smallPipe2: TypedPipe[(T, V, Double)])
      extends PMIDataContainer[T, U, V](smallPipe1, smallPipe2) {
    override private[pmi] def joinFeatureSumsToPipe1(
      regularizedPipe: TypedPipe[(T, U, Double)],
      featureSums: TypedPipe[(U, Double)]
    )(
      implicit ordering: Ordering[U],
      serializer: U => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[U])] = {
      getDataWithSumsHash(regularizedPipe, featureSums)
    }

    override private[pmi] def joinFeatureSumsToPipe2(
      regularizedPipe: TypedPipe[(T, V, Double)],
      featureSums: TypedPipe[(V, Double)]
    )(
      implicit ordering: Ordering[V],
      serializer: V => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[V])] = {
      getDataWithSumsHash(regularizedPipe, featureSums)
    }
  }

  case class SymmetricPMIContainer[T, U](
    data: TypedPipe[(T, U, Double)]) {
    def joinFeatureSumsToPipe(
      regularizedPipe: TypedPipe[(T, U, Double)],
      featureSums: TypedPipe[(U, Double)],
    )(
      implicit ordering: Ordering[U],
      serializer: U => Array[Byte]
    ): TypedPipe[(T, PMI.FeatureStats[U])] = {
      getDataWithSumsSketch(regularizedPipe, featureSums)
    }
  }
}
