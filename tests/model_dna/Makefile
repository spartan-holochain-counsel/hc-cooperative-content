
TARGET_DIR			= target/wasm32-unknown-unknown/debug

zomes/%.wasm:			$(TARGET_DIR)/%.wasm
	cp $< $@
$(TARGET_DIR)/%.wasm:		Makefile zomes/%/src/*.rs zomes/%/Cargo.toml
	@echo "Building '$*' WASM: $@"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=../target cargo build \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
