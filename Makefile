#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

BUILD ?= build

build_dir := $(BUILD)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(build_dir)

microkit_board := qemu_virt_aarch64
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
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target-dir $(build_dir)/target \
			--out-dir $(build_dir) \
			--target aarch64-sel4-microkit-minimal \
			--release \
			-p $(1)

endef

crate_names := \
	banscii-artist \
	banscii-assistant \
	banscii-pl011-driver

crates := $(foreach crate_name,$(crate_names),$(call crate,$(crate_name)))

$(eval $(foreach crate_name,$(crate_names),$(call build_crate,$(crate_name))))

### Loader

system_description := banscii.system

loader := $(build_dir)/loader.img

$(loader): $(system_description) $(crates)
	$(MICROKIT_SDK)/bin/microkit \
		$< \
		--search-path $(build_dir) \
		--board $(microkit_board) \
		--config $(microkit_config) \
		-r $(build_dir)/report.txt \
		-o $@

### Run

qemu_cmd := \
	qemu-system-aarch64 \
		-machine virt -cpu cortex-a53 -m size=2G \
		-serial mon:stdio \
		-nographic \
		-device loader,file=$(loader),addr=0x70000000,cpu-num=0

.PHONY: run
run: $(loader)
	$(qemu_cmd)

.PHONY: test
test: test.py $(loader)
	python3 $< $(qemu_cmd)
