package com.twitter.media_understanding.model_proxy.config

import com.twitter.decider.Decider
import com.twitter.decider.NullDecider
import com.twitter.inject.annotations.Flag
import com.twitter.media_understanding.common.config.FeatureRegistry
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class Features @Inject() (
  decider: Decider,
  @Flag("use-devel-feature-config") useDevelFeatureConfig: Boolean,
) extends FeatureRegistry(decider, useDevelFeatureConfig) {
  val loadShedding = feature("enable_load_shedding").unavailableOnDevel

  val adultContent = feature("disable_adult_content_v1").disabling.availableOnDevel
  val xXNsfwModelV1 = feature("disable_xx_nsfw_v1").disabling.availableOnDevel
  val fingerprintingModelV1 = feature("disable_fingerprinting_v1").disabling.availableOnDevel
  val fingerprintingModelV2 = feature("disable_fingerprinting_v2").disabling.availableOnDevel
  val categoryClassificationModelV1 = feature(
    "disable_category_classification_v1").disabling.availableOnDevel
  val categoryClassificationEmbeddingsModelV1 = feature(
    "disable_category_classification_embeddings_v1").disabling.availableOnDevel
  val hatefulSymbolRecognitionModelV1 = feature(
    "disable_hateful_symbol_recognition_v1").disabling.availableOnDevel
  val violenceAndGoreModelV1 = feature("disable_violence_and_gore_v1").disabling.availableOnDevel

  val gifsForCategoryClassification = feature(
    "enable_gifs_for_category_classification").availableOnDevel
  val gifsForCategoryClassificationEmbeddings = feature(
    "enable_gifs_for_category_classification_embeddings").availableOnDevel

  val dummyCategoryClassificationEmbeddings = feature(
    "enable_dummy_category_classification_embeddings").availableOnDevel
  val categoryClassificationEmbeddingsAltDarkTraffic = feature(
    "enable_category_classification_embeddings_alt_dark_traffic").availableOnDevel
}

class NullFeatures extends Features(new NullDecider, false)
