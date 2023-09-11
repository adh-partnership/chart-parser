#!/usr/bin/env bash

set -ex

HUB=${HUB:-adhp}
IMAGE=${IMAGE:-chart-parser}
TAG=${TAG:-latest}

cargo build --release
docker buildx build -t "${HUB}/${IMAGE}:${TAG}" .

if [[ -z ${DRY_RUN:-} ]]; then
  docker push "${HUB}/${IMAGE}:${TAG}"
fi
