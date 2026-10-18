package com.twitter.media_understanding.model_proxy.services

import com.twitter.finagle.stats.StatsReceiver
import com.twitter.media_understanding.model_proxy.exception.MediaNotFoundException
import com.twitter.mediaservices.commons.BlobStoreClient
import com.twitter.util.Future
import java.nio.ByteBuffer

class BlobStoreService(
  blobStoreClient: BlobStoreClient,
  statsReceiver: StatsReceiver) {

  private[this] val scopedStats = statsReceiver.scope("annotate_getMediaByPath")
  private[this] val successCounter = scopedStats.counter("success")
  private[this] val failureCounter = scopedStats.counter("failure")
  private[this] val totalCounter = scopedStats.counter("total")

  def getMediaByPath(mediaPath: String): Future[ByteBuffer] = {
    blobStoreClient
      .get(mediaPath).flatMap {
        case Some(response) => Future.value(response.data)
        case None =>
          Future.exception(new MediaNotFoundException(s"${mediaPath} not found in blobstore"))
      }.onFailure { _ =>
        failureCounter.incr()
      }.onSuccess { _ =>
        successCounter.incr()
      }.ensure {
        totalCounter.incr()
      }
  }
}
