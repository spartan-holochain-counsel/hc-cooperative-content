
SOURCE_FILES		= %/dna.yaml %/Makefile \
				%/zomes/Cargo.lock %/zomes/*/Cargo.toml %/zomes/*/src/*.rs \
				%/test_types/Cargo.toml %/test_types/src/*.rs \
				../Makefile ../zomes/Cargo.* ../zomes/*/Cargo.toml ../zomes/*/src/*.rs \
				../coop_content_types/Cargo.toml ../coop_content_types/src/*.rs \
				../hdk_extensions/Cargo.toml ../hdk_extensions/src/*.rs

%.dna:			$(SOURCE_FILES)
	rm -f $@
	@cd $*; find zomes -maxdepth 1 -type d ! -path zomes -exec make {}.wasm \;
	@echo -e "\x1b[37mPackaging DNA '$@'\x1b[0m"
	@hc dna pack -o $@ $* 2> >(head -n 1 | sed 's/.*/\x1b[31;1m&\x1b[0m/')
