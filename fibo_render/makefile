run: build
run:
	cargo run --release

debug: build
debug:
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