

%.dna:				%/dna.yaml %/Makefile %/zomes/* %/zomes/*/* %/zomes/*/*/*
	rm -f $@
	@cd $*; find zomes -maxdepth 1 -type d ! -path zomes -exec make {}.wasm \;
	@echo -e "\x1b[37mPackaging DNA '$@'\x1b[0m"
	@hc dna pack -o $@ $* 2> >(head -n 1 | sed 's/.*/\x1b[31;1m&\x1b[0m/')
