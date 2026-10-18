package com.twitter.hub.pmi

import com.twitter.hub.pmi.PMI.CrossedPMIStats
import com.twitter.hub.pmi.math.Math._
import com.twitter.hub.pmi.math.MathParams
import com.twitter.scalding.TypedPipe

object PMISetExtractors {

  case class EntityCount[T](entityId: T, count: Long)

  case class PMISetOutput[T, U, V](dataRow: (T, U, Double), label: V, pmi: Double)

  def getEntityCounts[K](
    data: TypedPipe[(K, Double)]
  )(
    implicit ordering: Ordering[K]
  ): TypedPipe[EntityCount[K]] = {
    data.group.size
      .map { case (entityId, count) => EntityCount(entityId, count) }
  }

  def getPMISetPerEntityAndLabel[T, U, V, W](
    data: TypedPipe[(T, U, Double)],
    model: TypedPipe[CrossedPMIStats[U, V]],
    onlyIncludeLabels: Option[Set[V]],
    numReducers: Int = 10000
  )(
    implicit orderingU: Ordering[U],
    serialize: U => Array[Byte]
  ): TypedPipe[PMISetOutput[T, U, V]] = {
    val filteredModel = onlyIncludeLabels
      .map(filterFeatures => model.filter(pmiStats => filterFeatures.contains(pmiStats.featureId2)))
      .getOrElse(model)

    data
      .groupBy(_._2)
      .sketch(numReducers)
      .join(filteredModel.groupBy(_.featureId1))
      .mapValues {
        case (dataRow, pmiStats) =>
          PMISetOutput(dataRow, pmiStats.featureId2, pmiStats.pmi)
      }
      .values
  }

  def getLeaveOneOutPMIExpSetPerEntityAndLabel[T, U, V, W](
    data1: TypedPipe[(T, U, Double)],
    data2: TypedPipe[(T, V, Double)],
    model: TypedPipe[CrossedPMIStats[U, V]],
    onlyIncludeLabels: Option[Set[V]],
    numReducers: Int = 10000
  )(
    implicit orderingT: Ordering[T],
    orderingU: Ordering[U],
    orderingV: Ordering[V],
    mathParams: MathParams,
    serialize: U => Array[Byte]
  ): TypedPipe[PMISetOutput[T, U, V]] = {
    val filteredData2 = onlyIncludeLabels
      .map(filterFeatures =>
        data2.filter {
          case (instanceId @ _, featureId, value @ _) => filterFeatures.contains(featureId)
        })
      .getOrElse(data2)

    val filteredModel = onlyIncludeLabels
      .map(filterFeatures => model.filter(pmiStats => filterFeatures.contains(pmiStats.featureId2)))
      .getOrElse(model)

    data1
      .groupBy(_._2)
      .sketch(numReducers)
      .join(filteredModel.groupBy(_.featureId1))
      .values
      .groupBy {
        case ((instanceId, featureId @ _, value @ _), pmiStats) => (instanceId, pmiStats.featureId2)
      }
      .leftJoin(filteredData2.groupBy {
        case (instanceId, featureId, _) => (instanceId, featureId)
      })
      .flatMapValues {
        case (((instanceId, featureId, value), pmiStats), optData2) =>
          val pmi = optData2
            .map {
              case (instanceId2 @ _, featureId2 @ _, value2) =>
                val prod = value * value2
                val prodSq = prod * prod
                val rawPMI = computePMIExpWithVariance(
                  pmiStats.sumProd - prod,
                  pmiStats.sumProdSq - prodSq,
                  pmiStats.featureSum1 - value,
                  pmiStats.featureSum2 - value2,
                  pmiStats.totalInstances - 1
                )
                getSparsePMI(rawPMI)
            }
            .getOrElse(Some(pmiStats.pmi))

          pmi.map { finalPmi =>
            PMISetOutput((instanceId, featureId, value), pmiStats.featureId2, finalPmi)
          }
      }
      .values
  }
}
