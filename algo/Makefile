WARNINGS := -Wall -Wextra -pedantic -Wshadow -Wpointer-arith -Wcast-align \
            -Wwrite-strings -Wmissing-prototypes -Wmissing-declarations \
            -Wredundant-decls -Wnested-externs -Winline -Wno-long-long \
            -Wstrict-prototypes
			#-Wconversion

CFLAGS=-lpthread $(WARNINGS) -Isrc/ 
LDFLAGS=-lpthread ${WARNINGS} 

RELEASE_FLAGS=-O3 -march=native -mtune=native
DEBUG_FLAGS=-ggdb -g3 -O0 -fno-inline

SOURCES=
SOURCES+=external/c_threads_lib/thread_pool/thread_pool.c
SOURCES+=external/c_threads_lib/blocking_queue/linked_queue/linked_queue.c 
SOURCES+=external/c_threads_lib/blocking_queue/blocking_queue.c
SOURCES+=lib_fibo_jump.c


OBJECTS  := $(patsubst %.c,build/%.o,$(SOURCES))
DEPFILES := $(patsubst %.o,%.d,$(OBJECTS))

.PHONY: all debug help clean mpropper

all: CFLAGS+=$(RELEASE_FLAGS)
all: LDFLAGS+=-s $(RELEASE_FLAGS)
all: src/external/gmp-6.3.0/.libs/libgmp.a fibo_calc

-include $(DEPFILES)

debug: CFLAGS+=$(DEBUG_FLAGS)
debug: LDFLAGS+=$(DEBUG_FLAGS)
debug: src/external/gmp-6.3.0/.libs/libgmp.a fibo_calc

help:
	@echo "TLDR; Makefile help"
	@echo "    make             Compiles and links the program (stripped)"
	@echo "    make debug       Compiles and links the program with debugging flags"
	@echo "    make clean       Cleans compilation temporary files (slow down subsequent ones)"
	@echo "    make mpropper    Cleans compilation files and executable"
	@echo "    make help        Displays this help"

fibo_calc: $(OBJECTS) src/external/gmp-6.3.0/.libs/libgmp.a
	@$(CC) $(OBJECTS) $(LDFLAGS) -lgmp -I src/external/gmp-6.3.0/.libs -o fibo_calc
	@echo "La compilation a réussie! Le programme s'apelle fibo_calc, lancez le en terminal avec la commande ./fibo_calc"

build:
	@mkdir -p build/external/c_threads_lib/thread_pool
	@mkdir -p build/external/c_threads_lib/blocking_queue/linked_queue


build/%.o: src/%.c Makefile build
	$(CC) $(CFLAGS) -MMD -MP -c $< -o $@

clean:
	-rm -rf build

mpropper: clean
	-rm fibo_calc
	-rm -rf src/external/gmp-6.3.0

src/external/gmp-6.3.0/.libs/libgmp.a:
	cd src/external && \
	rm -rf gmp-6.3.0 && \
	tar -xvf gmp-6.3.0.tar.xz
	cd src/external/gmp-6.3.0/ && \
	export CFLAGS="-O3 -march=native -mtune=native" && \
	./configure --enable-static=yes --enable-shared=no --disable-cxx && \
	make
