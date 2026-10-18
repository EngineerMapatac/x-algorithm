#!/bin/bash


function usage {
  echo "Usage: $0 [--trace] [service options]"
  echo
  echo "  --trace          set the tracing log level"
  echo "  --prod           run against production Media Model Proxy service (default is staging)"
  echo "  --prod-mp        run against production Media Platform services (default is staging)"
  echo "  --help           this help"
  echo
  exit 1
}

LOG_LEVEL="INFO"
ENV="staging"
MP_ENV="staging"
MEDIA_PATH_PREFIX="twitter_test"

while [ -n "$1" ]; do
  case "$1" in
    --trace)
      LOG_LEVEL="TRACE"
      shift
      ;;
    --prod)
      ENV="prod"
      shift
      ;;
    --prod-mp)
      MP_ENV="prod"
      MEDIA_PATH_PREFIX=""
      shift
      ;;
    --help)
      usage
      exit
      ;;
    *)
      break
      ;;
  esac
done

source "$(git rev-parse --show-toplevel)/devprod/source-sh-setup"

source "$(git rev-parse --show-toplevel)/twitter-server-internal/scripts/socks.sh"

set -o nounset
set -eu

if [ ! -f "${HOME}/.s2s/local/devel/integrationtest/${USER}/client.key" ]; then
  if command -v developer-cert-util > /dev/null; then
    echo "Generating local S2S certificates"
    developer-cert-util --job integrationtest --local --client-only
  else
    echo "developer-cert-util was not found, you need to install it from MDE"
    exit 0
  fi
fi

DIST_DIR="${REPO_ROOT}/dist"
IMAGES_DIR="${DIST_DIR}/integration-images"

if [ ! -d "${IMAGES_DIR}" ]; then
  echo "Downloading integration images package"
  (
    cd "${DIST_DIR}" &&
    packer fetch --cluster=smf1 media-model-proxy integration-images latest &&
    unzip integration-images.zip > /dev/null
  )
fi

DTAB_ADD=($(IFS=$'\n' cat << EOF
  /srv=>/$/nil
  /srv=>/srv#/staging
  /s/media-model-proxy/media-model-proxy=>/cluster/local/media-model-proxy/${ENV}/media-model-proxy
  /s/photurkey/mediainfo=>/cluster/local/photurkey/${MP_ENV}/mediainfo
  /s/image-fetcher-service/image_fetcher=>/cluster/local/image-fetcher-service/${MP_ENV}/image_fetcher
  /s/user-image-service/uis=>/cluster/local/user-image-service/${MP_ENV}/uis
  /s/media-analysis-service/mas=>/cluster/local/media-analysis-service/${MP_ENV}/mas
  /s/test-user-service/test-user-service=>/cluster/local/test-user-service/prod/test-user-service
EOF))
DTAB_ADD=$(IFS=';' ; echo "${DTAB_ADD[*]}")


JVM_OPTIONS=($(IFS=$'\n' cat << EOF
  -DsocksProxyHost=localhost
  -DsocksProxyPort=50001
  -Dcom.twitter.server.resolverZkHosts=localhost:2181
  -Dcom.twitter.finagle.util.loadServiceDenied=com.twitter.finagle.stats.OstrichStatsReceiver,com.twitter.finagle.stats.CommonsStatsReceiver
  -Dlogback.configurationFile=logback-local.xml
  -Dlog.level="${LOG_LEVEL}"
  -Dorg.apache.thrift.readLength=1572864
  -Ddtab.add=${DTAB_ADD}
  -Dmedia-model-proxy.disable=false
  -Dservice.identifier=${USER}:integrationtest:devel:local
  -Dthrift.clientId=media-model-proxy.test
  -Dimages-dir=${IMAGES_DIR}
  -Dmedia-path-prefix=${MEDIA_PATH_PREFIX}
EOF))

BAZEL_OPTIONS=($(IFS=$'\n' cat << EOF
--test_timeout=900
--test_output=all
--test_env=RUN_LIVETESTS=1
EOF))

TARGET="media-understanding/media-model-proxy/integrationtest/src/test/scala/com/twitter/media_understanding/model_proxy/integrationtest"

CMD=(./bazel test --test_arg="--jvm_flags=${JVM_OPTIONS[*]}" "${BAZEL_OPTIONS[@]}" ${TARGET})

echo "Running:"
echo "${CMD[@]}"

"${CMD[@]}"
