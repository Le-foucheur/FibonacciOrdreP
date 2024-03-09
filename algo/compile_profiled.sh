#STEPS=1

PROFILE_DIR=$(pwd)/pgo/
rm -rf PROFILE_DIR
mkdir -p PROFILE_DIR
CFLAGS_PROFILE_GEN="-fprofile-generate=$PROFILE_DIR -Wno-error=coverage-mismatch -fprofile-arcs -fvpt"
CFLAGS_PROFILE_USE="-fprofile-use -fprofile-dir=$PROFILE_DIR -Wno-error=coverage-mismatch -fprofile-correction"
LDFLAGS_PROFILE_GEN="-fprofile-arcs"

OLD_CFLAGS="$CFLAGS"
OLD_LDFLAGS="$LDFLAGS"


make clean
CFLAGS="$OLD_CFLAGS $CFLAGS_PROFILE_GEN" LDFLAGS="$OLD_LDFLAGS $LDFLAGS_PROFILE_GEN" make "$1" || exit 1
time ./test.sh
#for _ in $(seq "$STEPS")
#do
#make clean
#CFLAGS="$OLD_CFLAGS $CFLAGS_PROFILE_GEN $CFLAGS_PROFILE_USE" LDFLAGS="$OLD_LDFLAGS $LDFLAGS_PROFILE_GEN" make "$1" || exit 1
#time ./test.sh
#done
make clean
CFLAGS="$OLD_CFLAGS $CFLAGS_PROFILE_USE" LDFLAGS="$OLD_LDFLAGS" make "$1"
CFLAGS="$OLD_CFLAGS $CFLAGS_PROFILE_USE" LDFLAGS="$OLD_LDFLAGS" make "$1"_archive
time ./test.sh
