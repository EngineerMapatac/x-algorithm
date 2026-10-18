package com.twitter.botmaker.runtime

import com.twitter.eventbus.client.kafka.K8sKafkaDest
import org.apache.kafka.common.config.SslConfigs

object TwitterKafkaUtil {

  def normalizeDest(dest: String, tlsEnabled: Boolean): String = {
    if (K8sKafkaDest.isK8sDest(dest)) {
      dest
    } else if (tlsEnabled && !dest.endsWith(":kafka-tls")) {
      s"$dest:kafka-tls"
    } else {
      dest.split(":").head
    }
  }

  def k8sTlsConfigOverrides(dest: String, tlsEnabled: Boolean): Map[String, String] = {
    if (tlsEnabled && K8sKafkaDest.isK8sDest(dest)) {
      Map(
        SslConfigs.SSL_TRUSTSTORE_LOCATION_CONFIG -> K8sKafkaDest.TruststoreLocation,
        SslConfigs.SSL_ENDPOINT_IDENTIFICATION_ALGORITHM_CONFIG -> ""
      )
    } else {
      Map.empty
    }
  }
}
