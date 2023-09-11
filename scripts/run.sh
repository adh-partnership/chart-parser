#!/bin/bash
#
# Copyright Daniel Hawton
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -ex

echo "$MISC_OPTIONS"

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

CONDITIONAL_HOST_MOUNTS="${CONDITIONAL_HOST_MOUNTS:-} "

# docker conditional host mount (needed for make docker push)
if [[ -d "${HOME}/.docker" ]]; then
  CONDITIONAL_HOST_MOUNTS+="--mount type=bind,source=${HOME}/.docker,destination=/config/.docker,readonly "
fi

# gitconfig conditional host mount (needed for git commands inside container)
if [[ -f "${HOME}/.gitconfig" ]]; then
  CONDITIONAL_HOST_MOUNTS+="--mount type=bind,source=${HOME}/.gitconfig,destination=/home/.gitconfig,readonly "
fi

# .netrc conditional host mount (needed for git commands inside container)
if [[ -f "${HOME}/.netrc" ]]; then
  CONDITIONAL_HOST_MOUNTS+="--mount type=bind,source=${HOME}/.netrc,destination=/home/.netrc,readonly "
fi

WD=$(dirname "$0")
WD=$(cd "$WD"; pwd)
IMG=adhp/build-tools:main-latest

MOUNT_SOURCE="${MOUNT_SOURCE:-${PWD}}"
MOUNT_DEST="${MOUNT_DEST:-/work}"

read -ra DOCKER_RUN_OPTIONS <<< "${DOCKER_RUN_OPTIONS:-}"

[[ -t 1 ]] && DOCKER_RUN_OPTIONS+=("-it")
DOCKER_GID="${DOCKER_GID:-$(grep '^docker:' /etc/group | cut -f3 -d:)}"
[[ ${UID} -ne 0 ]] && DOCKER_RUN_OPTIONS+=(-u "${UID}:${DOCKER_GID}")

MISC_OPTIONS="${MISC_OPTIONS:-}"

# shellcheck disable=SC2086
docker run \
    --rm \
    "${DOCKER_RUN_OPTIONS[@]}" \
    --init \
    --sig-proxy=true \
    ${DOCKER_SOCKET_MOUNT:--v /var/run/docker.sock:/var/run/docker.sock} \
    $CONTAINER_OPTIONS \
    $MISC_OPTIONS \
    -e IN_BUILD_CONTAINER=1 \
    -e TZ="${TIMEZONE:-$TZ}" \
    --mount "type=bind,source=${MOUNT_SOURCE},destination=/work" \
    --mount "type=volume,source=go,destination=/go" \
    --mount "type=volume,source=gocache,destination=/gocache" \
    --mount "type=volume,source=cache,destination=/home/.cache" \
    ${CONDITIONAL_HOST_MOUNTS} \
    -w "${MOUNT_DEST}" "${IMG}" "$@"