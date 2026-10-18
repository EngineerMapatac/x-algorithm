package com.twitter.hub.agatha.util

import com.twitter.hub.agatha.thriftscala.FeatureIdentifier
import com.twitter.hub.agatha.thriftscala.ModelledFeature
import com.twitter.hub.pmi.PMI.CrossedPMIStats

object CrossedPMIStatToModelledFeature {
  def apply(crossed: CrossedPMIStats[FeatureIdentifier, String]): ModelledFeature =
    ModelledFeature(
      crossed.featureId1,
      crossed.featureId2,
      crossed.sumProd,
      crossed.sumProdSq,
      crossed.featureSum1,
      crossed.featureSum2,
      crossed.totalInstances,
      crossed.pmi
    )

  def invert(mf: ModelledFeature): CrossedPMIStats[FeatureIdentifier, String] =
    CrossedPMIStats(
      mf.identifier,
      mf.label,
      mf.sumOfUnionProduct,
      mf.sumOfSquaredUnionProduct,
      mf.sumOfFeatureIdentifierValues,
      mf.sumOfLabelValues,
      mf.totalUniqueUserIds,
      mf.pmi
    )
}
