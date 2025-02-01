.PHONY:			FORCE
SHELL			= bash

TARGET			= release
TARGET_DIR		= target/wasm32-unknown-unknown/release

TYPES_DIR		= crates/hc_coop_content_types
SDK_DIR			= crates/hc_coop_content_sdk
INT_DIR			= zomes/coop_content
CSR_DIR			= zomes/coop_content_csr
COMMON_SOURCE_FILES	= Makefile Cargo.toml \
				$(TYPES_DIR)/Cargo.toml $(TYPES_DIR)/src/*.rs
INT_SOURCE_FILES	= $(COMMON_SOURCE_FILES) \
				$(INT_DIR)/Cargo.toml $(INT_DIR)/src/*.rs
CSR_SOURCE_FILES	= $(INT_SOURCE_FILES) \
				$(CSR_DIR)/Cargo.toml $(CSR_DIR)/src/*.rs \
				$(SDK_DIR)/Cargo.toml $(SDK_DIR)/src/*.rs

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

zomes:
	mkdir $@
$(COOP_CONTENT_WASM):
$(COOP_CONTENT_CSR_WASM):
zomes/%.wasm:			$(TARGET_DIR)/%.wasm
	@echo -e "\x1b[38;2mCopying WASM ($<) to 'zomes' directory: $@\x1b[0m"; \
	cp $< $@

$(TARGET_DIR)/%.wasm:		$(INT_SOURCE_FILES)
	rm -f zomes/$*.wasm
	@echo -e "\x1b[37mBuilding zome '$*' -> $@\x1b[0m"; \
	RUST_BACKTRACE=1 cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time

$(TARGET_DIR)/%_csr.wasm:	zomes $(CSR_SOURCE_FILES)
	rm -f zomes/$*_csr.wasm
	@echo -e "\x1b[37mBuilding zome '$*_csr' -> $@\x1b[0m";
	RUST_BACKTRACE=1 cargo build --release \
	    --target wasm32-unknown-unknown \
	    --package $*_csr
	@touch $@ # Cargo must have a cache somewhere because it doesn't update the file time

npm-reinstall-local:
	cd tests; npm uninstall $(NPM_PACKAGE); npm i --save $(LOCAL_PATH)
npm-reinstall-public:
	cd tests; npm uninstall $(NPM_PACKAGE); npm i --save $(NPM_PACKAGE)

npm-use-app-interface-client-public:
npm-use-app-interface-client-local:
npm-use-app-interface-client-%:
	NPM_PACKAGE=@spartan-hc/app-interface-client LOCAL_PATH=../../app-interface-client-js make npm-reinstall-$*

npm-use-backdrop-public:
npm-use-backdrop-local:
npm-use-backdrop-%:
	NPM_PACKAGE=@spartan-hc/holochain-backdrop LOCAL_PATH=../../node-backdrop make npm-reinstall-$*



#
# Packages
#
.cargo/credentials:
	mkdir -p .cargo
	cp ~/$@ $@

preview-types-crate:
	DEBUG_LEVEL=debug make -s test
	cargo publish --dry-run --allow-dirty -p hc_coop_content_types
	touch crates/hc_coop_content_types/src/lib.rs
publish-types-crate:		.cargo/credentials
	DEBUG_LEVEL=debug make -s test
	cargo publish -p hc_coop_content_types
	touch crates/hc_coop_content_types/src/lib.rs

preview-sdk-crate:
	DEBUG_LEVEL=debug make -s test
	cargo publish --dry-run --allow-dirty -p hc_coop_content_sdk
	touch crates/hc_coop_content_sdk/src/lib.rs
publish-sdk-crate:		.cargo/credentials
	DEBUG_LEVEL=debug make -s test
	cargo publish -p hc_coop_content_sdk
	touch crates/hc_coop_content_sdk/src/lib.rs



#
# Testing
#
DEBUG_LEVEL	       ?= warn
TEST_ENV_VARS		= LOG_LEVEL=$(DEBUG_LEVEL)
MOCHA_OPTS		= -n enable-source-maps -t 10000

reset:
	rm -f zomes/*.wasm
	rm -f tests/*.dna
	rm -f tests/zomes/*.wasm
tests/%.dna:			build FORCE
	cd tests; make $*.dna
test-setup:			tests/node_modules

test:
	make -s test-unit
	make -s test-integration

test-unit:			test-unit-coop_content
test-unit-%:
	cd zomes;		RUST_BACKTRACE=1 cargo test $* -- --nocapture

test-integration:
	make -s test-setup
	make -s test-general
	make -s test-minimal
	make -s test-external
	make -s test-model
	make -s test-content-types

GENERAL_DNA			= tests/general_dna.dna
MINIMAL_DNA			= tests/minimal_dna.dna
MODEL_DNA			= tests/model_dna.dna
TEST_DNAS			= $(GENERAL_DNA) $(MINIMAL_DNA) $(MODEL_DNA)

test-general:			test-setup build $(GENERAL_DNA)
	cd tests; $(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) integration/test_general_dna.js
test-minimal:			test-setup build $(MINIMAL_DNA)
	cd tests; $(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) integration/test_minimal_dna.js
test-external:			test-setup build $(MINIMAL_DNA)
	cd tests; $(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) integration/test_minimal_external_pointers.js
test-model:			test-setup build $(TEST_DNAS)
	cd tests; $(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) integration/test_model_dna.js
test-content-types:		test-setup build $(TEST_DNAS)
	cd tests; $(TEST_ENV_VARS) npx mocha $(MOCHA_OPTS) integration/test_content_types.js



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

PRE_EDITION = edition = "2018"
NEW_EDITION = edition = "2021"

PRE_HDIE_VERSION = whi_hdi_extensions = "0.13"
NEW_HDIE_VERSION = whi_hdi_extensions = "0.14"

PRE_HDKE_VERSION = whi_hdk_extensions = "0.13"
NEW_HDKE_VERSION = whi_hdk_extensions = "0.14"


GG_REPLACE_LOCATIONS = ':(exclude)*.lock' zomes/*/ crates/*/ tests/zomes

update-hdk-extensions-version:
	git grep -l '$(PRE_HDKE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDKE_VERSION)|$(NEW_HDKE_VERSION)|g'
update-hdi-extensions-version:
	git grep -l '$(PRE_HDIE_VERSION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's|$(PRE_HDIE_VERSION)|$(NEW_HDIE_VERSION)|g'
update-edition:
	git grep -l '$(PRE_EDITION)' -- $(GG_REPLACE_LOCATIONS) | xargs sed -i 's/$(PRE_EDITION)/$(NEW_EDITION)/g'
reset-locks:
	rm -f Cargo.lock
	rm -f tests/zomes/Cargo.lock


#
# Documentation
#
TYPES_DOCS		= target/doc/coop_content_types/index.html
SDK_DOCS		= target/doc/coop_content_sdk/index.html
COOP_DOCS		= target/doc/coop_content/index.html
COOP_CSR_DOCS		= target/doc/coop_content_csr/index.html

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
$(COOP_DOCS):
$(COOP_CSR_DOCS):
target/doc/%/index.html:	zomes/*/src/**
	cd zomes; cargo test --doc
	cd zomes; cargo doc
	@echo -e "\x1b[37mOpen docs in file://$(shell pwd)/$@\x1b[0m";
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



#
# NPM packaging
#
prepare-zomelets-package:	zomelets/node_modules
	cd zomelets; rm -f dist/*
	cd zomelets; npx webpack
	cd zomelets; MODE=production npx webpack
	cd zomelets; gzip -kf dist/*.js
preview-zomelets-package:	clean-files prepare-zomelets-package
	DEBUG_LEVEL=trace make -s test
	cd zomelets; npm pack --dry-run .
create-zomelets-package:	clean-files prepare-zomelets-package
	DEBUG_LEVEL=trace make -s test
	cd zomelets; npm pack .
publish-zomelets-package:	clean-files prepare-zomelets-package
	DEBUG_LEVEL=trace make -s test
	cd zomelets; npm publish --access public .
