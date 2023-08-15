work_root := ..

image_tag := podhrmic/rust-sel4-sel4cp-demo
dockerfile := Dockerfile

.PHONY: none
none:

.PHONY: build
build:
	sudo docker build \
		-t $(image_tag) -f $(dockerfile) .

.PHONY: run
run: build
	sudo docker run -it -w /work -v $(abspath $(work_root)):/work  $(image_tag)
