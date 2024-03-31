all: help

graphic: build
graphic:
	cargo run --release --features="graphic"

headless: build
	cargo run --release

debug: build
	cargo run

build:
	cd ../algo && make mod2_archive \
	&& cp fibo_mod2.a ../fibo_render/libfibo_mod2.a \
	&& cp src/external/gmp-6.3.0/.libs/libgmp.a ../fibo_render/libgmp.a

clean:
	cd ../algo && make clean
	rm -f libfibo_mod2.a
	rm -f libgmp.a
	rm -f gnom.out perf.data perf.data.old

mpropper: clean
mpropper:
	cd ../algo && make mpropper
	cargo clean

help:
	@echo "TLDR; Makefile help"
	@echo "    make graphic       Run fibo_render in GUI mode"
	@echo "    make headless      Run fibo_render in headless mode"
	@echo "    make debug  		  Run fibo_render in dev mode (Headless by default)"
	@echo "    make build         Build GMP and computation lib, and copy .a files"
	@echo "    make clean         Cleans compilation temporary files (Without Rust libs)"
	@echo "    make mpropper     Cleans compilation temporary files (With Rust libs)"
	@echo "    make help          Displays this help"