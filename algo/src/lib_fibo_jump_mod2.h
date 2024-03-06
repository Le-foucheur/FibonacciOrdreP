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

#ifndef __AVX512F__
#ifndef __AVX2__
#ifdef __SSE4_1__
#warning "Your CPU do not support AVX2, slow code (using SSE4.1 only) will be used"
#else //SSE
#warning "Your CPU do not support AVX2, nor SSE4.1, slow code will be used"
#endif //SSE
#else //AVX2
#warning "Your CPU do not support AVX512, slow code (using AVX2 only) will be used"
#endif //AVX2
#endif //AVX512

#if BYTE_ORDER == LITTLE_ENDIAN

//due to endianness, cond_reg[1,2] contain the tests in the folowing order:
    // (23),(01) = 3,2,1,0 (64 bits pack,into 32) ... so we rotate as: 
    //          dest   3 2 1 0    the first time to put them back in order
    //           src   0 1 2 3    then we rotate by symply tacking the next one
    #define ROTATOR1 0b00011011
    #define ROTATOR2 0b00111001
//AVX2 implementation problem

#define NORMAL _reverse
struct char8 {
    char char8;
    char char7;
    char char6;
    char char5;
    char char4;
    char char3;
    char char2;
    char char1;
};
#elif BYTE_ORDER == BIG_ENDIAN
// cond_reg[1,2] contain the tests in the folowing order: (maybe? need testing)
    // 3,2,1,0 (64 bits pack) ... so we rotate as: 
    //          dest   3 2 1 0    the first time to put them back in order
    //           src   0 1 2 3    then we rotate by symply tacking the next one
    #define ROTATOR1 0b00011011
    #define ROTATOR2 0b00111001


#define NORMAL 
struct char8 {
    char char1;
    char char2;
    char char3;
    char char4;
    char char5;
    char char6;
    char char7;
    char char8;
};
#else
  #error "Ce programme ne supporte que le big et le little endian"
#endif

typedef struct char8 char8 ;
static_assert(sizeof(char8)==sizeof(uint64_t), "There is uncontrolled padding or oversized uuint64_t ...");

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
void arr_setb(unsigned char* array,size_t index,bool set);

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

#endif // LIB_FIBO_JUMP2_H
