import com.twitter.cortex_media_annotator.thriftscala._
import com.twitter.decider.Decider
import com.twitter.finagle.stats.StatsReceiver
import com.twitter.media_understanding.model_proxy.config.Config
import com.twitter.media_understanding.model_proxy.util.ScribeLogger
import com.twitter.ml.api.thriftscala.DataRecord
import com.twitter.util.Await
import org.scalatest.BeforeAndAfterEach
import org.scalatest.FunSuite
import org.scalatestplus.mockito.MockitoSugar

class ScribeLoggerSpec extends FunSuite with MockitoSugar with BeforeAndAfterEach {

  val category = "category"
  val config = () => mock[Config]
  val decider = mock[Decider]
  val statsReceiver = mock[StatsReceiver]

  val defaultBlobstorePath = "path"
  val defaultMediaId = "id"

  val scribeLogger = new ScribeLogger(category, config, decider, statsReceiver)

  test("scribe should work without a category") {
    val request = AnnotationRequest(
      entity = MediaEntity(
        category = None,
        blobstorePath = Some(defaultBlobstorePath),
        mediaId = Some(defaultMediaId)
      ),
      models = Set(MediaModel.Fingerprinting)
    )
    val response = AnnotationResponse(
      datarecord = Map.empty[MediaModel, DataRecord],
      annotations = Some(
        Map(
          MediaModel.Fingerprinting -> Seq.empty
        )
      )
    )

    Await.result(scribeLogger.scribe(request, response))
  }
}
