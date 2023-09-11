RUN=./scripts/run.sh
MAKE_CONTAINER=$(RUN) make --no-print-directory -e -f Makefile.core.mk

%:
	@$(MAKE_CONTAINER) $@

default:
	@$(MAKE_CONTAINER)

shell:
	@$(RUN) /bin/bash

.PHONY: docker
docker:
	@bash scripts/docker_build.sh

.PHONY: docker-push
docker-push:
	@bash scripts/docker_build.sh --push

.PHONY: default shell