WARNINGS := -Wall -Wextra -pedantic -Wshadow -Wpointer-arith -Wcast-align \
            -Wwrite-strings -Wmissing-prototypes -Wmissing-declarations \
            -Wredundant-decls -Wnested-externs -Winline -Wno-long-long \
            -Wstrict-prototypes -Werror=odr -Werror=lto-type-mismatch -Werror=strict-aliasing -Wstrict-overflow=2
			#-Wconversion

CFLAGS+=-lpthread -pthread $(WARNINGS) -Isrc/ --std=gnu2x
LDFLAGS+=-lpthread -pthread ${WARNINGS}
#--static

RELEASE_FLAGS=-O3 -march=native -mtune=native -flto -fwhole-program -fomit-frame-pointer -fopenmp
# -fopenmp
DEBUG_FLAGS=-ggdb -g3 -O0 -fno-inline -march=native

SOURCES=
SOURCES+=external/C-Thread-Pool/thpool.c
SOURCES+=lib_fibo_jump.c
SOURCES+=demo.c
OBJECTS+=$(patsubst %.c,build/%.o,$(SOURCES))

SOURCES_M2=
SOURCES_M2+=external/C-Thread-Pool/thpool.c
SOURCES_M2+=lib_fibo_jump_mod2.c
SOURCES_M2+=demo_mod2.c
OBJECTS_M2+=$(patsubst %.c,build/%.o,$(SOURCES_M2))


.PHONY: all main main_prog main_debug mod2 mod2_debug mod2_prog debug help clean mpropper main_archive mod2_archive

all: help

-include build/external/C-Thread-Pool/thpool.d
-include build/*.d

main: CFLAGS+=$(RELEASE_FLAGS)
main: LDFLAGS+=-s $(RELEASE_FLAGS)
main: main_prog

main_prog: src/external/gmp-6.3.0/.libs/libgmp.a fibo_calc


main_archive: CFLAGS+=$(RELEASE_FLAGS)
main_archive: LDFLAGS+=-s $(RELEASE_FLAGS)
main_archive: src/external/gmp-6.3.0/.libs/libgmp.a fibo.a

main_debug: CFLAGS+=$(DEBUG_FLAGS)
main_debug: LDFLAGS+=$(DEBUG_FLAGS)
main_debug: main_prog

mod2: CFLAGS+=$(RELEASE_FLAGS)
mod2: LDFLAGS+=-s $(RELEASE_FLAGS)
mod2: mod2_prog

mod2_prog: CFLAGS+=-std=gnu2x
mod2_prog: src/external/gmp-6.3.0/.libs/libgmp.a fibo_calc_mod2

mod2_archive: CFLAGS+=$(RELEASE_FLAGS)
mod2_archive: LDFLAGS+=-s $(RELEASE_FLAGS)
mod2_archive: src/external/gmp-6.3.0/.libs/libgmp.a fibo_mod2.a


mod2_debug: CFLAGS+=$(DEBUG_FLAGS)
mod2_debug: LDFLAGS+=$(DEBUG_FLAGS)
mod2_debug: mod2_prog


help:
	@echo "TLDR; Makefile help"
	@echo "    make main          Compiles and links the calculation program (stripped)"
	@echo "    make main_debug    Compiles and links the calculation program with debugging flags"
	@echo "    make main_archive  Compiles and produce library for static linkage"
	@echo "    make mod2          Compiles and links the calculation program (stripped)"
	@echo "    make mod2_debug    Compiles and links the calculation program with debugging flags"
	@echo "    make mod2_archive  Compiles and produce library for static linkage"
	@echo "    make clean         Cleans compilation temporary files (slow down subsequent ones)"
	@echo "    make mpropper      Cleans compilation files and executable and gmp compiled files"
	@echo "    make help          Displays this help"
	@echo "Special compilation flags: pass them using CFLAGS=\"Flags\" make [target]"
	@echo "   -D FIBO_NO_AVX      disable AVX and AVX512 implementations"
	@echo "   -D FIBO_NO_AVX512   disable AVX512 implementation"
	@echo "   -D FIBO_NO_SSE      disable AVX and SSE implementation"
	

fibo_calc: $(OBJECTS) src/external/gmp-6.3.0/.libs/libgmp.a
	$(CC) $^ $(LDFLAGS) -I src/external/gmp-6.3.0/.libs -o fibo_calc
	@echo "La compilation a réussie! Le programme s'apelle fibo_calc, lancez le en terminal avec la commande ./fibo_calc"

fibo_calc_mod2: $(OBJECTS_M2) src/external/gmp-6.3.0/.libs/libgmp.a
	$(CC) $(LDFLAGS) -I src/external/gmp-6.3.0/.libs -o fibo_calc_mod2 $^
	@echo "La compilation a réussie! Le programme s'apelle fibo_calc_mod2, lancez le en terminal avec la commande ./fibo_calc_mod2"

fibo_mod2.a: $(OBJECTS_M2)
	ar ru $@ $^
	ranlib $@

fibo.a: $(OBJECTS)
	ar ru $@ $^
	ranlib $@

build:
	@mkdir -p build/external/C-Thread-Pool


build/%.o: src/%.c Makefile build
	$(CC) $(CFLAGS) -MMD -MP -c $< -o $@

clean:
	-rm -rf build

mpropper: clean
	-rm fibo_calc
	-rm fibo_calc_mod2
	-rm -rf src/external/gmp-6.3.0

src/external/gmp-6.3.0/.libs/libgmp.a:
	cd src/external && \
	rm -rf gmp-6.3.0 && \
	tar -xvf gmp-6.3.0.tar.xz
	cd src/external/gmp-6.3.0/ && \
	export CFLAGS="-O3 -march=native -mtune=native" && \
	./configure --enable-static=yes --enable-shared=no --disable-cxx CFLAGS="-std=gnu99 -O3 -march=native -mtune=native" && \
	make

src/external/C-Thread-Pool/thpool.c:
	cd src/external && \
	git clone https://github.com/Pithikos/C-Thread-Pool/
