

%.dna:				%/dna.yaml %/Makefile %/zomes/* %/zomes/*/* %/zomes/*/*/*
	@cd $*; find zomes -maxdepth 1 -type d ! -path zomes -exec make {}.wasm \;
	@echo "Packaging '$*': $@"
	hc dna pack -o $@ $*
