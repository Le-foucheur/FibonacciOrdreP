STEPS=3

PROFILE_DIR=$(pwd)/pgo/
rm -rf PROFILE_DIR
mkdir -p PROFILE_DIR
CFLAGS_PROFILE_GEN="-fprofile-generate=${PROFILE_DIR} -Wno-error=coverage-mismatch -fprofile-arcs -fvpt"
CFLAGS_PROFILE_USE="-fprofile-use=${PROFILE_DIR} -Wno-error=coverage-mismatch -fprofile-correction"
LDFLAGS_PROFILE_GEN="-fprofile-arcs"

OLD_CFLAGS="$CFLAGS"
OLD_LDFLAGS="$LDFLAGS"


if [[ "$1" == "mod2" ]]; then
PROGRAM=fibo_calc_mod2
else
  if [[ "$1" == "main" ]]; then
  PROGRAM=fibo_calc
  else
    echo "unrecognized arg, exiting"
    exit 1
  fi
fi


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
