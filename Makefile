
.PHONY:			FORCE
SHELL			= bash
TARGET			= release
TARGET_DIR		= target/wasm32-unknown-unknown/release
SOURCE_FILES		= Makefile zomes/Cargo.* zomes/*/Cargo.toml zomes/*/src/*.rs zomes/*/src/*/* \
				coop_content_sdk/Cargo.toml coop_content_sdk/src/*.rs

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
# Packages
#
.cargo/credentials:
	cp ~/$@ $@

preview-types-crate:		test-debug
	cd coop_content_types; cargo publish --dry-run --allow-dirty
	touch coop_content_types/src/lib.rs
publish-types-crate:		test-debug .cargo/credentials
	cd coop_content_types; cargo publish
	touch coop_content_types/src/lib.rs

preview-sdk-crate:		test-debug
	cd coop_content_sdk; cargo publish --dry-run --allow-dirty
	touch coop_content_sdk/src/lib.rs
publish-sdk-crate:		test-debug .cargo/credentials
	cd coop_content_sdk; cargo publish
	touch coop_content_sdk/src/lib.rs



#
# Testing
#
reset:
	rm -f zomes/*.wasm
	rm -f tests/*.dna
	rm -f tests/zomes/*.wasm
tests/%.dna:			build FORCE
	cd tests; make $*.dna
test-setup:			tests/node_modules

test:				test-unit test-integration
test-debug:			test-unit test-integration-debug

test-unit:			test-unit-coop_content
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

test-integration:		test-setup	\
				test-general	\
				test-minimal	\
				test-external	\
				test-model
test-integration-debug:		test-setup		\
				test-general-debug	\
				test-minimal-debug	\
				test-external-debug	\
				test-model-debug

GENERAL_DNA			= tests/general_dna.dna
MINIMAL_DNA			= tests/minimal_dna.dna
MODEL_DNA			= tests/model_dna.dna
TEST_DNAS			= $(GENERAL_DNA) $(MINIMAL_DNA) $(MODEL_DNA)

test-general:			test-setup build $(GENERAL_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_general_dna.js
test-general-debug:		test-setup build $(GENERAL_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_general_dna.js

test-minimal:			test-setup build $(MINIMAL_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_minimal_dna.js
test-minimal-debug:		test-setup build $(MINIMAL_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_minimal_dna.js

test-external:			test-setup build $(MINIMAL_DNA)
	cd tests; RUST_LOG=none LOG_LEVEL=fatal npx mocha integration/test_minimal_external_pointers.js
test-external-debug:		test-setup build $(MINIMAL_DNA)
	cd tests; RUST_LOG=info LOG_LEVEL=trace npx mocha integration/test_minimal_external_pointers.js

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

PRE_HDIE_VERSION = whi_hdi_extensions = "=0.3.0"
NEW_HDIE_VERSION = whi_hdi_extensions = "0.4"

PRE_HDKE_VERSION = whi_hdk_extensions = "0.2"
NEW_HDKE_VERSION = whi_hdk_extensions = "0.4"

GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/ *_types/ *_sdk/ tests/zomes

update-hdk-extensions-version:
	git grep -l '$(PRE_HDKE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDKE_VERSION)|$(NEW_HDKE_VERSION)|g'
update-hdi-extensions-version:
	git grep -l '$(PRE_HDIE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDIE_VERSION)|$(NEW_HDIE_VERSION)|g'


#
# Documentation
#
TYPES_DOCS		= target/doc/coop_content_types/index.html
SDK_DOCS		= target/doc/coop_content_sdk/index.html
COOP_DOCS		= target/doc/coop_content/index.html

$(TYPES_DOCS):		coop_content_types/src/**
	cd coop_content_types; cargo test --doc
	cd zomes; cargo doc
	touch coop_content_types/src/lib.rs
	@echo -e "\x1b[37mOpen docs in file://$(shell pwd)/$(TYPES_DOCS)\x1b[0m";
$(SDK_DOCS):		coop_content_sdk/src/**
	cd coop_content_sdk; cargo test --doc
	cd zomes; cargo doc
	touch coop_content_sdk/src/lib.rs
	@echo -e "\x1b[37mOpen docs in file://$(shell pwd)/$(SDK_DOCS)\x1b[0m";
$(COOP_DOCS):		zomes/*/src/**
	cd zomes; cargo test --doc
	cd zomes; cargo doc
	@echo -e "\x1b[37mOpen docs in file://$(shell pwd)/$(COOP_DOCS)\x1b[0m";
docs:			$(SDK_DOCS) $(COOP_DOCS)
docs-watch:
	@inotifywait -r -m -e modify		\
		--includei '.*\.rs'		\
			zomes/			\
			coop_content_sdk	\
	| while read -r dir event file; do	\
		echo -e "\x1b[37m$$event $$dir$$file\x1b[0m";\
		make docs;			\
	done
