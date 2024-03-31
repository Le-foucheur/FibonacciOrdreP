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

#ifdef FIBO_NO_AVX
  #define FIBO_NO_AVX512
#endif

#if BYTE_ORDER == LITTLE_ENDIAN
//#define NORMAL _reverse
#define INDEX_MULT 1
#define INDEX_FLAT 0

#elif BYTE_ORDER == BIG_ENDIAN
//#define NORMAL
#define INDEX_MULT (-1)
#define INDEX_FLAT 1

#else
  #error "Ce programme ne supporte que le big et le little endian"
#endif

//autocompletion purpose: comment when building/releasing
//#define __AVX__
#define __AVX512F__
//#define FIBO_AVX512_TEST

#if defined (__AVX512F__) && (!defined (FIBO_NO_AVX512)) && (!defined (FIBO_NO_SSE))
//#if defined(FIBO_AVX512_TEST)
//#define FIBO_IMPLEM 'T'
//#else //for Test
#define FIBO_IMPLEM '5'
//#endif //for AVX*5*12
#else
#if defined(__AVX__) && (!defined (FIBO_NO_AVX)) && (!defined (FIBO_NO_SSE))

#define FIBO_IMPLEM '2'
#else //for avx*2*
#if defined (__SSE2__) && (!defined (FIBO_NO_SSE))
#define FIBO_IMPLEM 'S'
#else  //for SSE2
#define FIBO_IMPLEM 'i'
#endif //for Ints only
#endif
#endif

typedef unsigned char cond_t;

#if FIBO_IMPLEM == 'i'
//int64 only
#warning "Your CPU do not support AVX2, slow code will be used"

  typedef uint64_t bytes_t;
  #define byte_zero 0
  #define get_bytes arr_geti
  #define arr_set_result arr_set7c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 7
#endif
#if FIBO_IMPLEM == 'S'
  typedef __m128i bytes_t;
  #define byte_zero _mm_setzero_si128 ()
  #define get_bytes arr_get2i
  #define arr_set_result arr_set15c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 15
  
#endif
#if FIBO_IMPLEM == '2'
  #warning "Your CPU do not support AVX512, slow code (using AVX2 only) will be used"

  //AVX2 specific function
  __m256i arr_get4i(unsigned char* array,ptrdiff_t index);
  __attribute__((always_inline)) inline void arr_set31c(unsigned char* array,ptrdiff_t base_index,__m256i value);
  __m256i arr_broadload(unsigned char* array,ptrdiff_t index);

  typedef __m256i bytes_t;
  #define byte_zero _mm256_setzero_si256()
  #define get_bytes arr_get4i
  #define arr_set_result arr_set31c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 31
#endif

#if FIBO_IMPLEM == '5'
//******************************* AVX512 new test *************************************************
  __m512i arr_get8i(unsigned char* array,ptrdiff_t index);
  
  typedef __m512i bytes_t;
  
  #define byte_zero _mm512_setzero_epi32()
  #define get_bytes arr_get8i
  #define arr_set_result arr_set63c
  //number of bytes treated as once in one jump_formula call
  #define BATCH_SIZE 63
#endif
typedef struct {
      bytes_t part0;
      bytes_t part1;
      bytes_t part2;
      bytes_t part3;
      bytes_t part4;
      bytes_t part5;
      bytes_t part6;
      bytes_t part7;
} accumulator;

static_assert(8==sizeof(uint64_t), "There is uncontrolled padding or oversized uuint64_t ...");

int getNumCores(void);

unsigned char* arr_get_false_addr(unsigned char* real_addr,size_t size);
unsigned char* arr_get_real_addr(unsigned char* array,size_t size);
unsigned char* array_create(size_t size);
void array_free(unsigned char* array,size_t size);
unsigned char* array_realoc(unsigned char* array,size_t old_size,size_t new_size);
unsigned char arr_getc(unsigned char* array,ptrdiff_t index);
uint64_t arr_geti(unsigned char* array,ptrdiff_t index);
void arr_setc(unsigned char* array,ptrdiff_t index,unsigned char set);
void arr_seti(unsigned char* array,ptrdiff_t index,uint64_t set);
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
/* Initialize nescessary structure to use with p up to p_arg.
  Return 1 on error, 0 otherwise */
int fibo_mod2_initialization(size_t p_arg);
#endif // LIB_FIBO_JUMP2_H
