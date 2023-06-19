
.PHONY:			FORCE
SHELL			= bash
TARGET			= release
TARGET_DIR		= target/wasm32-unknown-unknown/release
SOURCE_FILES		= Makefile zomes/Cargo.* zomes/*/Cargo.toml zomes/*/src/*.rs zomes/*/src/*/* \
				coop_content_types/Cargo.toml coop_content_types/src/*.rs \
				hdi_extensions/Cargo.toml hdi_extensions/src/*.rs \
				hdk_extensions/Cargo.toml hdk_extensions/src/*.rs

# Zomes (WASM)
COOP_CONTENT_WASM	= zomes/coop_content.wasm
COOP_CONTENT_CSR_WASM	= zomes/coop_content_csr.wasm


#
# Project
#
tests/package-lock.json:	tests/package.json
	touch $@
tests/node_modules:		tests/package-lock.json
	cd tests; \
	npm install
	touch $@
clean:
	rm -rf \
	    tests/node_modules \
	    .cargo \
	    target

rebuild:			clean build
build:				$(COOP_CONTENT_WASM) $(COOP_CONTENT_CSR_WASM)

zomes/%.wasm:			zomes/$(TARGET_DIR)/%.wasm
	@echo -e "\x1b[38;2mCopying WASM ($<) to 'zomes' directory: $@\x1b[0m"; \
	cp $< $@
zomes/$(TARGET_DIR)/%.wasm:	$(SOURCE_FILES)
	rm -f zomes/$*.wasm
	@echo -e "\x1b[37mBuilding zome '$*' -> $@\x1b[0m"; \
	cd zomes; \
	RUST_BACKTRACE=1 CARGO_TARGET_DIR=target cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time

use-local-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev ../../node-holochain-backdrop/
use-npm-backdrop:
	cd tests; npm uninstall @whi/holochain-backdrop
	cd tests; npm install --save-dev @whi/holochain-backdrop



#
# Testing
#
tests/%.dna:			build FORCE
	cd tests; make $*.dna
test-setup:			tests/node_modules

test:				test-unit test-integration		test-e2e
test-debug:			test-unit test-integration-debug	test-e2e-debug

test-unit:			test-unit-coop_content
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

test-integration:		test-setup test-model
test-integration-debug:		test-setup test-model-debug

MODEL_DNA			= tests/model_dna.dna
TEST_DNAS			= $(MODEL_DNA)

test-model:			test-setup build $(TEST_DNAS)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_model_dna.js
test-model-debug:		test-setup build $(TEST_DNAS)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_model_dna.js



#
# Repository
#
clean-remove-chaff:
	@find . -name '*~' -exec rm {} \;
clean-files:		clean-remove-chaff
	git clean -nd
clean-files-force:	clean-remove-chaff
	git clean -fd
clean-files-all:	clean-remove-chaff
	git clean -ndx
clean-files-all-force:	clean-remove-chaff
	git clean -fdx

PRE_HDK_VERSION = "0.3.0-beta-dev.2"
NEW_HDK_VERSION = ""

PRE_HDI_VERSION = "0.4.0-beta-dev.1"
NEW_HDI_VERSION = ""

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/ *_types/ hc_utils

update-hdk-version:
	git grep -l $(PRE_HDK_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDK_VERSION)/$(NEW_HDK_VERSION)/g'
update-hdi-version:
	git grep -l $(PRE_HDI_VERSION) -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_HDI_VERSION)/$(NEW_HDI_VERSION)/g'
