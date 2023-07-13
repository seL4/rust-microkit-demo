build_dir := build

PLATFORM ?= QEMU

ifeq ($(PLATFORM),QEMU)
sel4cp_board := qemu_arm_virt
system_description := banscii.system
crates := \
	banscii-artist \
	banscii-assistant \
	banscii-pl011-driver \
	uart-interface-types \
	eth-driver \
	ethernet-interface-types \
	eth-client
else
sel4cp_board := zcu102
system_description := banscii_zcu102.system
crates := \
	banscii-artist \
	banscii-assistant \
	uart-driver \
	uart-interface-types \
	eth-driver \
	ethernet-interface-types \
	eth-client \
	timer
endif

sel4cp_config := debug
sel4cp_sdk_config_dir := $(SEL4CP_SDK)/board/$(sel4cp_board)/$(sel4cp_config)

.PHONY: none
none:

.PHONY: clean
clean:
	rm -rf $(build_dir)

### Protection domains

rust_target_path := support/targets
rust_sel4cp_target := aarch64-sel4cp-minimal
target_dir := $(build_dir)/target

common_env := \
	RUST_TARGET_PATH=$(abspath $(rust_target_path)) \
	SEL4_INCLUDE_DIRS=$(abspath $(sel4cp_sdk_config_dir)/include)

common_options := \
	-Z build-std=core,alloc,compiler_builtins \
	-Z build-std-features=compiler-builtins-mem \
	--target $(rust_sel4cp_target) \
	--release \
	--target-dir $(abspath $(target_dir)) \
	--out-dir $(abspath $(build_dir))

target_for_crate = $(build_dir)/$(1).elf
intermediate_target_for_crate = $(build_dir)/$(1).intermediate

define build_crate

$(target_for_crate): $(intermediate_target_for_crate)

.INTERMDIATE: $(intermediate_target_for_crate)
$(intermediate_target_for_crate):
	$$(common_env) \
		cargo build \
			$$(common_options) \
			-p $(1)

endef

built_crates := $(foreach crate,$(crates),$(call target_for_crate,$(crate)))

$(eval $(foreach crate,$(crates),$(call build_crate,$(crate))))

# C components
uartps.elf:
	BUILD_DIR=$(abspath $(build_dir)) \
	SEL4CP_SDK=$(abspath $(SEL4CP_SDK)) \
	SEL4CP_BOARD=$(sel4cp_board) \
	SEL4CP_CONFIG=debug \
	make -C crates/uartps clean
	BUILD_DIR=$(abspath $(build_dir)) \
	SEL4CP_SDK=$(abspath $(SEL4CP_SDK)) \
	SEL4CP_BOARD=$(sel4cp_board) \
	SEL4CP_CONFIG=debug \
	make -C crates/uartps uartps.elf
	mv crates/uartps/uartps.elf $(build_dir)/.

### Loader

loader := $(build_dir)/loader.img

$(loader): $(system_description) $(built_crates)
	$(SEL4CP_SDK)/bin/sel4cp \
		$< \
		--search-path $(build_dir) \
		--board $(sel4cp_board) \
		--config $(sel4cp_config) \
		-r $(build_dir)/report.txt \
		-o $@

compile: $(loader)
	echo "Done!"

.PHONY: run
run:
	qemu-system-aarch64 \
		-machine virt \
		-cpu cortex-a53 -m size=1G \
		-device loader,file=$(loader),addr=0x70000000,cpu-num=0 \
		-serial mon:stdio \
		-nographic
