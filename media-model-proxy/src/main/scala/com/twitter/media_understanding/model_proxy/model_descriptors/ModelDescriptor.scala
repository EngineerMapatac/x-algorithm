package com.twitter.media_understanding.model_proxy.model_descriptors

import com.twitter.cortex_media_annotator.thriftscala.MediaModel
import com.twitter.decider.Feature
import com.twitter.media_understanding.model_proxy.clients.ModelClient
import com.twitter.mediaservices.commons.thriftscala.MediaCategory
import com.twitter.mediaservices.commons.util.MediaCategoryUtil
import com.twitter.media_understanding.model_proxy.exception.MediaCategoryNotSupportedException
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.util.Return
import com.twitter.util.Throw
import com.twitter.util.Try
import java.nio.ByteBuffer

abstract class ModelDescriptor {

  val model: MediaModel

  def unavailableCategories: Seq[(MediaCategory, Feature)] = Seq()

  def features(mediaCategory: Option[MediaCategory], media: ByteBuffer): Try[DataRecord]

  def serviceBuilders(): List[ModelClient.Builder]

  def labels(): Map[String, Long] = Map()

  def validate(dataRecord: DataRecord): Unit = {}

  def supportedCategories: Set[MediaCategory] = Set()

  def translateModelSupportedMediaCategory(mediaCategory: MediaCategory): Try[MediaCategory] = {
    if (supportedCategories.contains(mediaCategory)) {
      Return(mediaCategory)
    } else if (MediaCategoryUtil.isGif(mediaCategory)) {
      Return(MediaCategory.TweetGif)
    } else if (MediaCategoryUtil.isVideo(mediaCategory)) {
      Return(MediaCategory.TweetVideo)
    } else if (MediaCategoryUtil.isImage(mediaCategory)) {
      Return(MediaCategory.TweetImage)
    } else {
      Throw(
        new MediaCategoryNotSupportedException(
          s"${model.name} model does not support media category $mediaCategory"
        ))
    }
  }
}
