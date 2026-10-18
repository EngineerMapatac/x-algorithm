package com.twitter.visibility.under_the_hood

import com.twitter.gizmoduck.thriftscala.Labels
import java.nio.charset.StandardCharsets
import org.apache.thrift.protocol.TJSONProtocol
import org.apache.thrift.transport.TMemoryInputTransport
import scala.util.control.NonFatal

object GizmoduckLabelDiff {

  def labelApplyAndExpireTimes(json: String): Option[Map[String, (Long, Long)]] =
    try {
      val transport = new TMemoryInputTransport(json.getBytes(StandardCharsets.UTF_8))
      Some(
        Labels
          .decode(new TJSONProtocol(transport)).labels.iterator
          .map(l =>
            l.labelValue.name -> ((l.createdAtMsec, l.expiresAtMsec.getOrElse(Long.MaxValue))))
          .toMap)
    } catch {
      case NonFatal(_) => None
    }
}
