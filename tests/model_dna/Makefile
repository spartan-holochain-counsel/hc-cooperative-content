
TARGET_DIR			= target/wasm32-unknown-unknown/debug

zomes/%.wasm:			$(TARGET_DIR)/%.wasm
	@echo -e "\x1b[38;2mCopying WASM ($<) to 'zomes' directory: $@\x1b[0m"; \
	cp $< $@
$(TARGET_DIR)/%.wasm:		Makefile zomes/%/src/*.rs zomes/%/Cargo.toml
	rm -f zomes/$*.wasm
	@echo -e "\x1b[37mBuilding zome '$*' -> $@\x1b[0m"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=../target cargo build \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time
