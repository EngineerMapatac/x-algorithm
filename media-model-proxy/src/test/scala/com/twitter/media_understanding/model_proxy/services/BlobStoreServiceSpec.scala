package com.twitter.media_understanding.model_proxy.services

import com.twitter.finagle.stats.InMemoryStatsReceiver
import com.twitter.media_understanding.model_proxy.exception.MediaNotFoundException
import com.twitter.mediaservices.commons.BlobStoreClient
import com.twitter.mediaservices.commons.GetResponse
import com.twitter.util.Await
import com.twitter.util.Future
import java.nio.ByteBuffer
import org.mockito.Mockito._
import org.scalatest.FunSuite
import org.scalatestplus.mockito.MockitoSugar

class BlobStoreServiceSpec extends FunSuite with MockitoSugar {
  val mockBlobStoreClient = mock[BlobStoreClient]
  val statsReceiver = new InMemoryStatsReceiver
  val blobStoreService = new BlobStoreService(mockBlobStoreClient, new InMemoryStatsReceiver)
  val mockGetResponse = mock[GetResponse]
  val path = "mediaPath"

  test("getMediaByPath correctly returns bytes of media given its path in blobstore") {
    val byteBuff = ByteBuffer.wrap("test".getBytes)

    when(mockBlobStoreClient.get(path)).thenReturn(Future.value(Some(mockGetResponse)))
    when(mockGetResponse.data).thenReturn(byteBuff)

    assert(Await.result(blobStoreService.getMediaByPath(path)) == byteBuff)
  }

  test("getMediaByPath throws MediaNotFoundException returns if media not found in blobstore") {
    when(mockBlobStoreClient.get(path)).thenReturn(Future.value(None))

    intercept[MediaNotFoundException] {
      Await.result(blobStoreService.getMediaByPath(path))
    }
  }
}
