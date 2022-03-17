LINDERA_SERVER_VERSION ?= $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | select(.name=="lindera-server") | .version')

.DEFAULT_GOAL := build

clean:
	cargo clean

format:
	cargo fmt

build:
	cargo build --release --features=full

test:
	cargo test

tag:
	git tag v$(LINDERA_SERVER_VERSION)
	git push origin v$(LINDERA_SERVER_VERSION)

publish:
ifeq ($(shell curl -s -XGET https://crates.io/api/v1/crates/lindera-server | jq -r '.versions[].num' | grep $(LINDERA_SERVER_VERSION)),)
	(cd lindera-server && cargo package && cargo publish)
	sleep 10
endif

docker-build:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/linderamorphology/lindera-server/tags' | jq -r '."results"[]["name"]' | grep $(LINDERA_SERVER_VERSION)),)
	docker build --tag=linderamorphology/lindera-server:latest --build-arg="LINDERA_VERSION=$(LINDERA_SERVER_VERSION)" .
	docker tag linderamorphology/lindera-server:latest linderamorphology/lindera-server:$(LINDERA_SERVER_VERSION)
endif

docker-push:
ifeq ($(shell curl -s 'https://registry.hub.docker.com/v2/repositories/linderamorphology/lindera-server/tags' | jq -r '."results"[]["name"]' | grep $(LINDERA_SERVER_VERSION)),)
	docker push linderamorphology/lindera-server:latest
	docker push linderamorphology/lindera-server:$(LINDERA_SERVER_VERSION)
endif

docker-clean:
ifneq ($(shell docker ps -f 'status=exited' -q),)
	docker rm $(shell docker ps -f 'status=exited' -q)
endif
ifneq ($(shell docker images -f 'dangling=true' -q),)
	docker rmi -f $(shell docker images -f 'dangling=true' -q)
endif
ifneq ($(docker volume ls -f 'dangling=true' -q),)
	docker volume rm $(docker volume ls -f 'dangling=true' -q)
endif
