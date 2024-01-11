#!/bin/bash

set -e

pushd docker
docker build -t 'images_organizer_builder' .
popd

docker run \
  --rm \
  --network host \
  -v .:/project \
  -w /project/ \
  'images_organizer_builder' \
  /project/build_local.sh
