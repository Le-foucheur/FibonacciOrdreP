#ifndef LIB_FIBO_JUMP2_H
#define LIB_FIBO_JUMP2_H


#include <stdlib.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <endian.h>
#include <assert.h>
#include <limits.h>
#include <emmintrin.h>
#include <immintrin.h>
#include "external/gmp-6.3.0/gmp.h"


#ifndef __UINT64_TYPE__
  #error "This program was deeply optimzed for 64 bits, sry"
#endif
#define paste(a,b,c) a ## b ## c
#define concat(a,b,c) paste(a,b,c)

#define MASK8(index) (~ (1<<(index)))
#define MASK64(index) (~ (1ULL<<(index)))
#define MASK_LOW 0xFFFFFFFFFFFFFFULL
#define MASK_UP 0xFFFFFFFFFFFFFF00ULL

//autocompletion purpose, remove in production
//#define __AVX2__

#ifdef FIBO_NO_AVX
  #define FIBO_NO_AVX512
#endif

#if BYTE_ORDER == LITTLE_ENDIAN
#define NORMAL _reverse

#elif BYTE_ORDER == BIG_ENDIAN
#define NORMAL 

#else
  #error "Ce programme ne supporte que le big et le little endian"
#endif

//autocompletion purpose: comment when building/releasing
//#define __AVX2__
//#define __AVX512F__

#if  (!defined(__AVX512F__)) || defined(FIBO_NO_AVX512) 
#if (!defined(__AVX2__)) || defined (FIBO_NO_AVX) 

//int64 only
#warning "Your CPU do not support AVX2, slow code will be used"
  typedef struct {
      uint64_t part0;
      uint64_t part1;
      uint64_t part2;
      uint64_t part3;
      uint64_t part4;
      uint64_t part5;
      uint64_t part6;
      uint64_t part7;
  } accumulator;
  

  typedef uint64_t bytes_t;
  #define byte_zero 0
  #define get_bytes arr_geti
  #define arr_set_result arr_set7c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 7
#else //AVX2
  #warning "Your CPU do not support AVX512, slow code (using AVX2 only) will be used"

  typedef struct {
      __m256i part0;
      __m256i part1;
      __m256i part2;
      __m256i part3;
      __m256i part4;
      __m256i part5;
      __m256i part6;
      __m256i part7;
  } accumulator;


  //AVX2 specific function
  __m256i implem_array_get8i(unsigned char* array,ptrdiff_t index);
  __m256i implem_array_reverse_get8i(unsigned char* array,ptrdiff_t index);
  __attribute__((always_inline)) inline void arr_set31c(unsigned char* array,ptrdiff_t base_index,__m256i value);
  #define arr_get8i concat(implem_array, NORMAL, _get8i)
  typedef __m256i bytes_t;
  #define byte_zero _mm256_setzero_si256()
  #define get_bytes arr_get8i
  #define arr_set_result arr_set31c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 31



#endif //AVX2
#else //AVX512
  typedef __m512i accumulator ;
  typedef uint64_t bytes_t;
  #define byte_zero 0
  #define get_bytes arr_geti
  #define arr_set_result arr_set7c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 7
  

#endif

static_assert(8==sizeof(uint64_t), "There is uncontrolled padding or oversized uuint64_t ...");

int getNumCores(void);

unsigned char* implem_array_get_false_addr(unsigned char* real_addr,size_t size);
unsigned char* implem_array_reverse_get_false_addr(unsigned char* real_addr,size_t size);
#define arr_get_false_addr concat(implem_array, NORMAL, _get_false_addr)

unsigned char* implem_array_get_real_addr(unsigned char* array,size_t size);
unsigned char* implem_array_reverse_get_real_addr(unsigned char* array, size_t size);
#define arr_get_real_addr concat(implem_array, NORMAL, _get_real_addr)

unsigned char* array_create(size_t size);
void array_free(unsigned char* array,size_t size);
unsigned char* array_realoc(unsigned char* array,size_t old_size,size_t new_size);

unsigned char implem_array_getc(unsigned char* array,ptrdiff_t index);
unsigned char implem_array_reverse_getc(unsigned char* array,ptrdiff_t index);
#define arr_getc concat(implem_array, NORMAL, _getc)

uint64_t implem_array_geti(unsigned char* array,ptrdiff_t index);
uint64_t implem_array_reverse_geti(unsigned char* array,ptrdiff_t index);
#define arr_geti concat(implem_array, NORMAL, _geti)

void implem_array_setc(unsigned char* array,ptrdiff_t index,unsigned char set);
void implem_array_reverse_setc(unsigned char* array,ptrdiff_t index,unsigned char set);
#define arr_setc concat(implem_array, NORMAL, _setc)

void implem_array_seti(unsigned char* array,ptrdiff_t index,uint64_t set);
void implem_array_reverse_seti(unsigned char* array,ptrdiff_t index,uint64_t set);
#define arr_seti concat(implem_array, NORMAL, _seti)

bool char_getb(unsigned char ch,unsigned char index);
unsigned char char_setb(unsigned char ch,unsigned char index,bool set);
void arr_setb(unsigned char* array,ptrdiff_t index,bool set);

uint64_t int_setb(uint64_t it,unsigned char index,bool set);
bool int_getb(uint64_t it,unsigned char index);

bool arr_getb2(unsigned char* array,size_t arr_index,unsigned char c_index);

bool arr_getb(unsigned char* array,size_t index);

uint64_t arr_get7c(unsigned char* array,size_t index);
void arr_set7c(unsigned char* array,size_t index,uint64_t set);
uint64_t arr_get_unaligned_7c(unsigned char* array,size_t index,unsigned char shift);

/* initialize the thread pool used to calulate fibonacci numbers
  MUST be called once at program start

  Return non-zero on error
*/
int fibo2_init_thread_pool(size_t size);

/* Calcule le n-ième (et les p suivant) termes de la suite de fibonacci d'ordre p
*  Arguments: 
*    n: terme de la suite a calculer (pour des raisons d'optimisation, cette version ne supporte que les n>=0)
*    p: ordre de la suite de fibonacci (1 pour fibonaci standard) P doit etre au maximum ULL_MAX - 2. plus p est grand, plus les calculs consomeront de RAM
*    result: pointeur vers p+1 mpz_t valant Fp(n) a Fp(n+p)
*/
unsigned char* fibo_mod2(size_t p,mpz_t n);
//same, but passing as string for easier external usage without using gmp internally in other projects
//still need gmp linking
unsigned char* rust_fibo_mod2(size_t p_arg, const char* n);

#endif // LIB_FIBO_JUMP2_H
