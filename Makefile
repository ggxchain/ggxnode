# Variables
ENVIRONMENT ?=
IMAGE_TAG ?= latest
LATEST_COMMIT := $$(git rev-parse HEAD)
REGISTRY_HOST ?= ghcr.io/boostylabs
NODE_IMAGE_NAME = ggx-node

IMAGE_NODE_BACKUP = $(REGISTRY_HOST)/$(NODE_IMAGE_NAME)$(ENVIRONMENT):$(LATEST_COMMIT)
IMAGE_NODE_LATEST = $(REGISTRY_HOST)/$(NODE_IMAGE_NAME)$(ENVIRONMENT):$(IMAGE_TAG)

help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
%:
	@:

build_node: ## Build bridge-core docker image.
	DOCKER_BUILDKIT=1 docker build -f ./template/Dockerfile -t $(IMAGE_NODE_BACKUP) . && DOCKER_BUILDKIT=1 docker build -f ./template/Dockerfile -t $(IMAGE_NODE_LATEST) .

push_node: ## Push bridge-core docker image.
	docker push $(IMAGE_NODE_BACKUP) && docker push $(IMAGE_NODE_LATEST)

docker: ## Build and push all docker images.
	make build_node push_node
