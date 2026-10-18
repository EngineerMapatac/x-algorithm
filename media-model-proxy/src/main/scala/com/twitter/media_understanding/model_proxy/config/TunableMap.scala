package com.twitter.media_understanding.model_proxy.config

import com.twitter.util.tunable.ConfigbusTunableMap

class TunableMap
    extends ConfigbusTunableMap(
      serviceName = "media-understanding/media-model-proxy",
      id = Tunables.TunableId
    )
