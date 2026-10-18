#!/bin/bash


function usage {
  echo "Usage: $0 [--trace] [service options]"
  echo
  echo "  --trace                  set the tracing log level"
  echo "  --prod-mu                run against production Media Understanding services"
  echo "  --local-fingerprinting   add a dtab to connect to a local Fingerprinting model first"
  echo "  --help                   this help"
  echo
  exit 1
}

LOG_LEVEL="DEBUG"
MU_ENV="staging"
DTAB_EXTRA=("/dummy-entry=>/$/fail")

while [ -n "$1" ]; do
  case "$1" in
    --trace)
      LOG_LEVEL="TRACE"
      shift
      ;;
    --prod-mu)
      MU_ENV="prod"
      shift
      ;;
    --local-fingerprinting)
      DTAB_EXTRA+=(
        "/s/embeddings-category/magicpony-fingerprinting=>/$/fail"
        "/s/embeddings-category/magicpony-fingerprinting=>/$/inet/localhost/26101"
      )
      shift
      ;;
    --local-embeddings-category-classification)
      DTAB_EXTRA+=(
        "/s/embeddings-category/embeddings-category-classification=>/$/fail"
        "/s/embeddings-category/embeddings-category-classification=>/$/inet/localhost/26107"
      )
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

if [ ! -f "${HOME}/.s2s/local/devel/media-model-proxy/${USER}/client.key" ]; then
  if command -v developer-cert-util > /dev/null; then
    echo "Generating local S2S certificates"
    developer-cert-util --job media-model-proxy --local
  else
    echo "developer-cert-util was not found, you need to install it from MDE"
    exit 0
  fi
fi

CONFIG_ROOT="${REPO_ROOT}/media-understanding/media-model-proxy/config"

DTAB_ADD=($(IFS=$'\n' cat << EOF
  /srv=>/$/nil
  /srv=>/srv#/staging
  /s/adult-content/adult-content=>/cluster/local/adult-content/${MU_ENV}/adult-content
  /s/xx-nsfw/xx-nsfw=>/cluster/local/xx-nsfw/${MU_ENV}/xx-nsfw
  /s/magicpony-category/magicpony-category-classification=>/cluster/local/magicpony-category/${MU_ENV}/magicpony-category-classification
  /s/embeddings-category/embeddings-category-classification=>/cluster/local/embeddings-category/${MU_ENV}/embeddings-category-classification
  /s/magicpony-fingerprinting/magicpony-fingerprinting=>/cluster/local/magicpony-fingerprinting/${MU_ENV}/magicpony-fingerprinting
  /s/magicpony-hateful/magicpony-hateful-symbol-recognition=>/cluster/local/magicpony-hateful/${MU_ENV}/magicpony-hateful-symbol-recognition
  /s/blobstore=>/cluster/local/blobstore/prod
EOF))
DTAB_ADD=( "${DTAB_ADD[@]}" "${DTAB_EXTRA[@]}" )
DTAB_ADD=$(IFS=';' ; echo "${DTAB_ADD[*]}")

JVM_OPTIONS=($(IFS=$'\n' cat << EOF
  -DsocksProxyHost=localhost
  -DsocksProxyPort=50001
  -Dcom.twitter.server.resolverZkHosts=localhost:2181
  -Dcom.twitter.finagle.util.loadServiceDenied=com.twitter.finagle.stats.OstrichStatsReceiver,com.twitter.finagle.stats.CommonsStatsReceiver
  -Dlogback.configurationFile=logback-local.xml
  -Dlog.level=${LOG_LEVEL}
  -Dorg.apache.thrift.readLength=10000000
  -agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=27000
EOF))

JOB_OPTIONS=($(IFS=$'\n' cat << EOF
  -admin.port=:25000
  -thrift.port=:26000
  -decider.base=${CONFIG_ROOT}/decider.yml
  -decider.refreshBase=true
  -config.overlay=media-understanding/model-proxy/devel/${USER}/config_overlay.json
  -service.identifier=${USER}:media-model-proxy:devel:local
  -opportunistic.tls.level=required
  -diffy_proxy.enabled=false
  -dtab.add=${DTAB_ADD}
  -disable-admin-auth
  -use-devel-feature-config=true
EOF))

magicpony/tools/build_and_deploy.py \
  media-model-proxy media-understanding/media-model-proxy:bundle \
  --jvm --bundle --local --no-run --basename media-model-proxy-service-package-dist

JAR="${REPO_ROOT}/dist/media-model-proxy-service-package-dist-bundle/media-model-proxy.jar"

CMD=(java "${JVM_OPTIONS[@]}" -jar "${JAR}" "${JOB_OPTIONS[@]}" "$@")

echo "Running:"
echo "${CMD[@]}"

exec "${CMD[@]}"
