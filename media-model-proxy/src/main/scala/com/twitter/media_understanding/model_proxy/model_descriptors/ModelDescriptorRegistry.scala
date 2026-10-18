package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.inject.Injector
import javax.inject.Inject

class ModelDescriptorRegistry @Inject() (injector: Injector) {

  private[this] val descriptors: Map[MediaModel, ModelDescriptor] = Map(
    MediaModel.CategoryClassification -> injector.instance[CategoryClassificationModelDescriptor],
    MediaModel.CategoryClassificationEmbeddings -> injector
      .instance[CategoryClassificationEmbeddingsModelDescriptor],
    MediaModel.Fingerprinting -> injector.instance[FingerprintingModelDescriptor],
    MediaModel.FingerprintingV2 -> injector.instance[FingerprintingV2ModelDescriptor],
    MediaModel.HatefulSymbolRecognition -> injector
      .instance[HatefulSymbolRecognitionModelDescriptor],
    MediaModel.ViolenceAndGore -> injector.instance[ViolenceAndGoreModelDescriptor],
    MediaModel.XxNsfw -> injector.instance[XXNsfwModelDescriptor],
  )

  def getDescriptor(model: MediaModel): Option[ModelDescriptor] = {
    descriptors.get(model)
  }

  def getModels(): Set[MediaModel] = {
    descriptors.keySet
  }
}
