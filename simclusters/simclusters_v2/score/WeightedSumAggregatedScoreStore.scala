package com.twitter.simclusters_v2.score

import com.twitter.simclusters_v2.score.WeightedSumAggregatedScoreStore.WeightedSumAggregatedScoreParameter
import com.twitter.simclusters_v2.thriftscala.EmbeddingType
import com.twitter.simclusters_v2.thriftscala.GenericPairScoreId
import com.twitter.simclusters_v2.thriftscala.ModelVersion
import com.twitter.simclusters_v2.thriftscala.ScoreInternalId
import com.twitter.simclusters_v2.thriftscala.ScoringAlgorithm
import com.twitter.simclusters_v2.thriftscala.SimClustersEmbeddingId
import com.twitter.simclusters_v2.thriftscala.{Score => ThriftScore}
import com.twitter.simclusters_v2.thriftscala.{ScoreId => ThriftScoreId}
import com.twitter.simclusters_v2.thriftscala.{
  SimClustersEmbeddingPairScoreId => ThriftSimClustersEmbeddingPairScoreId
}
import com.twitter.util.Future

case class WeightedSumAggregatedScoreStore(parameters: Seq[WeightedSumAggregatedScoreParameter])
    extends AggregatedScoreStore {

  override def get(k: ThriftScoreId): Future[Option[ThriftScore]] = {
    val underlyingScores = parameters.map { parameter =>
      scoreFacadeStore
        .get(ThriftScoreId(parameter.scoreAlgorithm, parameter.idTransform(k.internalId)))
        .map(_.map(s => parameter.scoreTransform(s.score) * parameter.weight))
    }
    Future.collect(underlyingScores).map { scores =>
      if (scores.exists(_.nonEmpty)) {
        val newScore = scores.foldLeft(0.0) {
          case (sum, maybeScore) =>
            sum + maybeScore.getOrElse(0.0)
        }
        Some(ThriftScore(score = newScore))
      } else {
        None
      }
    }
  }
}

object WeightedSumAggregatedScoreStore {

  case class WeightedSumAggregatedScoreParameter(
    scoreAlgorithm: ScoringAlgorithm,
    weight: Double,
    idTransform: ScoreInternalId => ScoreInternalId,
    scoreTransform: Double => Double = identityScoreTransform)

  val SameTypeScoreInternalIdTransform: ScoreInternalId => ScoreInternalId = { id => id }
  val identityScoreTransform: Double => Double = { score => score }

  def genericPairScoreIdToSimClustersEmbeddingPairScoreId(
    embeddingType1: EmbeddingType,
    embeddingType2: EmbeddingType,
    modelVersion: ModelVersion
  ): ScoreInternalId => ScoreInternalId = {
    case id: ScoreInternalId.GenericPairScoreId =>
      ScoreInternalId.SimClustersEmbeddingPairScoreId(
        ThriftSimClustersEmbeddingPairScoreId(
          SimClustersEmbeddingId(embeddingType1, modelVersion, id.genericPairScoreId.id1),
          SimClustersEmbeddingId(embeddingType2, modelVersion, id.genericPairScoreId.id2)
        ))
  }

  val simClustersEmbeddingPairScoreIdToGenericPairScoreId: ScoreInternalId => ScoreInternalId = {
    case ScoreInternalId.SimClustersEmbeddingPairScoreId(simClustersId) =>
      ScoreInternalId.GenericPairScoreId(
        GenericPairScoreId(simClustersId.id1.internalId, simClustersId.id2.internalId))
  }
}
