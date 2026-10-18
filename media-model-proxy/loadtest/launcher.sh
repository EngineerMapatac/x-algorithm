#!/usr/bin/env bash


set -eu

REPO_ROOT="$(git rev-parse --show-toplevel)"

ENV=staging

DC=smf1

LOG_FILE=logs/media-model-proxy-sample.csv

ROLE=media-model-proxy

IAGO_SERVER_IDENTIFIER="${ROLE}:parrot-server:${ENV}:${DC}"
IAGO_FEEDER_IDENTIFIER="${ROLE}:parrot-feeder:${ENV}:${DC}"

if [ ! -f "media-understanding/media-model-proxy/loadtest/${LOG_FILE}" ]
then
  echo "ERROR: Input log file not found."
  echo
  echo "Check the loadtest documentation on how to obtain the log file."
  echo "https://example.invalid/media_understanding/services/media_model_proxy/service.html#load-testing"
  echo

  exit 1
fi

export CONFIG_FILE="config/loadtest.yaml"
export PANTS_BUNDLE_BASENAME="media-model-proxy-loadtest"
export PANTS_BUNDLE_TARGET="media-understanding/media-model-proxy/loadtest:bundle"
export JOB_NAME="media_model_proxy_loadtest"

"${REPO_ROOT}"/iago-internal/scripts/deploy.sh \
  -role=${ROLE} \
  -name=${JOB_NAME} \
  -env=${ENV} \
  -zone=${DC} \
  -pkg.name=${PANTS_BUNDLE_BASENAME} \
  -config=${CONFIG_FILE} \
  -s2s.id=${IAGO_SERVER_IDENTIFIER} \
  -instances=1 \
  -build=${PANTS_BUNDLE_TARGET}

open "https://example.invalid/iago"
open "https://example.invalid/${DC}/media-model-proxy/${ENV}"
