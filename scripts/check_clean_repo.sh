#!/usr/bin/env bash

if [[ -n $(git status --porcelain) ]]; then
  echo "Repo is not clean"
  git status
  git diff
  exit 1
fi
