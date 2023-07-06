work_root := ..

image_tag := rust-sel4-sel4cp-demo
dockerfile := Dockerfile

uid := $(shell id -u)
gid := $(shell id -g)

.PHONY: none
none:

.PHONY: build
build:
	sudo docker build \
		--build-arg UID=$(uid) --build-arg GID=$(gid) \
		-t $(image_tag) -f $(dockerfile) .

.PHONY: run
run: build
	sudo docker run -it -w /work -v $(abspath $(work_root)):/work  $(image_tag)
