#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

BUILD ?= build
BOARD ?= qemu_virt_aarch64

build_dir := $(BUILD)/$(BOARD)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(BUILD)

$(build_dir):
	mkdir -p $@

microkit_board := $(BOARD)
microkit_config := debug
microkit_sdk_config_dir := $(MICROKIT_SDK)/board/$(microkit_board)/$(microkit_config)

sel4_include_dirs := $(microkit_sdk_config_dir)/include

### Protection domains

crate = $(build_dir)/$(1).elf

define build_crate

$(crate): $(crate).intermediate

.INTERMDIATE: $(crate).intermediate
$(crate).intermediate:
	SEL4_INCLUDE_DIRS=$(abspath $(sel4_include_dirs)) \
		cargo build \
			--target-dir $(build_dir)/target \
			--artifact-dir $(build_dir) \
			--release \
			-p $(1) \
			$(extra-flags-$(1))

endef

crate_names := \
	banscii-artist \
	banscii-assistant \
	banscii-serial-driver

extra-flags-banscii-serial-driver := --features board-$(microkit_board)

crates := $(foreach crate_name,$(crate_names),$(call crate,$(crate_name)))

$(eval $(foreach crate_name,$(crate_names),$(call build_crate,$(crate_name))))

### Loader

system_description_template := banscii.system.template

system_description := $(build_dir)/banscii.system

$(system_description): generate_system_description.py $(system_description_template) | $(build_dir)
	python3 $< --template $(system_description_template) --board $(BOARD) -o $@

loader := $(build_dir)/loader.img

$(loader): $(system_description) $(crates)
	$(MICROKIT_SDK)/bin/microkit \
		$< \
		--search-path $(build_dir) \
		--board $(microkit_board) \
		--config $(microkit_config) \
		-r $(build_dir)/report.txt \
		-o $@

.PHONY: build
build: $(loader)

### Run

ifeq ($(BOARD),qemu_virt_aarch64)

qemu_cmd := \
	qemu-system-aarch64 \
		-machine virt,virtualization=on -cpu cortex-a53 -m size=2G \
		-serial mon:stdio \
		-nographic \
		-device loader,file=$(loader),addr=0x70000000,cpu-num=0

.PHONY: run
run: $(loader)
	$(qemu_cmd)

.PHONY: test
test: test.py $(loader)
	python3 $< $(qemu_cmd)

endif
