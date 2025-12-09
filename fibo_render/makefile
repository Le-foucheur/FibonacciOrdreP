all: help

graphic:
graphic:
	cargo run --release --features="graphic"

headless:
	cargo run --release

debug:
	cargo run

clean:
	rm -f gnom.out perf.data perf.data.old

mpropper: clean
mpropper:
	cargo clean

help:
	@echo "TLDR; Makefile help"
	@echo "    make graphic       Run fibo_render in GUI mode"
	@echo "    make headless      Run fibo_render in headless mode"
	@echo "    make debug  		  Run fibo_render in dev mode (Headless by default)"
	@echo "    make clean         Cleans compilation temporary files (Without Rust libs)"
	@echo "    make mpropper     Cleans compilation temporary files (With Rust libs)"
	@echo "    make help          Displays this help"
