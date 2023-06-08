
SOURCE_FILES		= Makefile zomes/%/Cargo.toml zomes/%/src/*.rs \
				test_types/Cargo.toml test_types/src/*.rs \
				../../Makefile ../../zomes/Cargo.* ../../zomes/*/Cargo.toml ../../zomes/*/src/*.rs \
				../../coop_content_types/Cargo.toml ../../coop_content_types/src/*.rs \
				../../hdk_extensions/Cargo.toml ../../hdk_extensions/src/*.rs
# MODE			= production
MODE			= development

ifeq ($(MODE), development)
    CARGO_OPTS		=
    TARGET_DIR		= target/wasm32-unknown-unknown/debug
else ifeq ($(MODE), production)
    CARGO_OPTS		= --release
    TARGET_DIR		= target/wasm32-unknown-unknown/release
else
    $(error Invalid MODE definition '$(MODE)'; expected development/production)
endif

zomes/%.wasm:			$(TARGET_DIR)/%.wasm
	@echo -e "\x1b[38;2mCopying WASM ($<) to 'zomes' directory: $@\x1b[0m"; \
	cp $< $@
$(TARGET_DIR)/%.wasm:		$(SOURCE_FILES)
	rm -f zomes/$*.wasm
	@echo -e "\x1b[37mBuilding zome '$*' -> $@\x1b[0m"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=../target cargo build $(CARGO_OPTS) \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
