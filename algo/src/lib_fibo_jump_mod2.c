#include "lib_fibo_jump_mod2.h"
#include "external/C-Thread-Pool/thpool.h"
#include <smmintrin.h>
#include <stddef.h>
#include <stdint.h>

#ifdef _WIN32
#include <windows.h>
#elif MACOS
#include <sys/param.h>
#include <sys/sysctl.h>
#else
#include <unistd.h>
#endif

//Protos:
void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,uint64_t result);
void jump_formula_plus1(void* k);
void jump_formula(void* k);
void refill_big_from_little(size_t last_valid);
void initialize_big(size_t last_valid,size_t init_max);
void add_one(void);
size_t mpz_get_siz(mpz_t z);

unsigned char* big_buffer;
size_t big_buffer_size;
unsigned char* little_buffer;
size_t little_buffer_size;
size_t p;

int getNumCores(void) {
#ifdef WIN32
    SYSTEM_INFO sysinfo;
    GetSystemInfo(&sysinfo);
    return sysinfo.dwNumberOfProcessors;
#elif MACOS
    int nm[2];
    size_t len = 4;
    uint32_t count;

    nm[0] = CTL_HW; nm[1] = HW_AVAILCPU;
    sysctl(nm, 2, &count, &len, NULL, 0);

    if(count < 1) {
        nm[1] = HW_NCPU;
        sysctl(nm, 2, &count, &len, NULL, 0);
        if(count < 1) { count = 1; }
    }
    return count;
#else
    return sysconf(_SC_NPROCESSORS_ONLN);
#endif
}

#define MIN(x,y) (x<y ? x : y)

threadpool calcul_pool;


int fibo2_init_thread_pool(size_t size){
  if (size==0) {
    size=getNumCores();
  }
  calcul_pool = thpool_init(size);
  if (calcul_pool==NULL) {
    return -1;
  }
  return 0;
}

//index are always from the end ...
//wich are used beetween normal or _reverse_ function depend on endianness of system (only big and litle endian support)

unsigned char* implem_array_get_false_addr(unsigned char* real_addr,size_t size){   return real_addr+size-1+8;}
unsigned char* implem_array_reverse_get_false_addr(unsigned char* real_addr,size_t size){    return real_addr+8;}
unsigned char* implem_array_get_real_addr(unsigned char* array,size_t size){  return array-size+1-8;}
unsigned char* implem_array_reverse_get_real_addr(unsigned char* array, size_t size){  return array-8;}

unsigned char* array_create(size_t size){  
  unsigned char* array = calloc(size+15,1);
  if (array==NULL) {
    return NULL;
  }
  return arr_get_false_addr(array,size);}
void array_free(unsigned char* array,size_t size){ if (array==NULL) return;  free(arr_get_real_addr(array,size));}
unsigned char* array_realoc(unsigned char* array,size_t old_size,size_t new_size){ return arr_get_false_addr(realloc(arr_get_real_addr(array,old_size), new_size),new_size);}



unsigned char implem_array_getc(unsigned char* array,ptrdiff_t index){  return array[-index];}
unsigned char implem_array_reverse_getc(unsigned char* array,ptrdiff_t index){  return array[index];}

uint64_t implem_array_geti(unsigned char* array,ptrdiff_t index){  return * ((uint64_t*)(array-index-7));}
uint64_t implem_array_reverse_geti(unsigned char* array,ptrdiff_t index){  return * ((uint64_t*)(array+index));}

void implem_array_setc(unsigned char* array,ptrdiff_t index,unsigned char set){  array[-index]=set;}
void implem_array_reverse_setc(unsigned char* array,ptrdiff_t index,unsigned char set){  array[index]=set;}

void implem_array_seti(unsigned char* array,ptrdiff_t index,uint64_t set){  *(uint64_t*)(array-index-7)=set;}
void implem_array_reverse_seti(unsigned char* array,ptrdiff_t index,uint64_t set){  *(uint64_t*)(array+index)=set;}

void arr_set7c(unsigned char* array,size_t index,uint64_t set){
  for (unsigned char i=0;i<7;i++){
    arr_setc(array,index+i,(unsigned char)((set>>(8*i))&0xFF));
  }
}

bool char_getb(unsigned char ch,unsigned char index){  return (bool)((ch>>index)&1);}
unsigned char char_setb(unsigned char ch,unsigned char index,bool set){  return (ch & MASK8(index)) | (set<<index);}

void arr_setb(unsigned char* array,size_t index,bool set){
  arr_setc(array,index>>3,char_setb(arr_getc(array,index>>3), index&0b111, set));
}

uint64_t int_setb(uint64_t it,unsigned char index,bool set){ return (it & MASK64(index)) | set<<index;}
bool int_getb(uint64_t it,unsigned char index){ return (bool)((it>>index)&1);}

bool arr_getb2(unsigned char* array,size_t arr_index,unsigned char c_index){  return char_getb(arr_getc(array,arr_index), c_index);}
bool arr_getb(unsigned char* array,size_t index){return arr_getb2(array, index>>3, (unsigned char)(index&0b111));}

#if  defined(__AVX512F__) && (!defined (FIBO_NO_AVX512))
//fastest AVX-512 implem
typedef __m512i accumulator ;
__attribute__((always_inline)) inline accumulator loop_once(accumulator acc,char condition, uint64_t bits);
__attribute__((always_inline)) inline uint64_t finalize(accumulator acc);

#define zero_acc() _mm512_setzero_epi32()
__attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,char condition, uint64_t bits){
  __m512i temp = _mm512_maskz_set1_epi64(condition,bits);
  return _mm512_xor_epi64(acc,temp);
}
__attribute__((always_inline)) inline
uint64_t finalize(accumulator acc){
  //due to inversion when using masks, acc contain bits to be shifted of 7,6,5,4,3,2,1,0, in this order
  __m256i temp1 = _mm512_extracti64x4_epi64 (acc, 0);
  temp1=_mm256_srli_epi64 (temp1, 4);
  __m256i temp2 = _mm512_extracti64x4_epi64 (acc, 1);
  temp2 = _mm256_xor_si256(temp1,temp2);  //contain to be shifted by 3,2,1,0
  
  __m128i temp3 = _mm256_extracti128_si256 (temp2, 1);  //contain 1,0
  temp2 = _mm256_srli_epi64(temp2,2);
  temp2 = _mm256_xor_si256 (temp2, _mm256_castsi128_si256(temp3));  //contain 1,0,useless,useless

  uint64_t result = _mm256_extract_epi64 (temp2, 1);
  result ^= _mm256_extract_epi64 (temp2, 0)>>1;
  return result;
}

void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,uint64_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  
  for (;i_base<=((ptrdiff_t)(p/8))-7;i_base+=7){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift);
  
    for (signed char i=6;i>=0;i--){
      unsigned char cond_bits_c =  (char)(cond_bits>>(8*i));
      uint64_t bits=arr_geti(big_buffer,ints_addr);
      accu = loop_once(accu, cond_bits_c, bits);      
      ints_addr++;
    }
    bit_addr-=7;
  }

  uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift);
  
  for (unsigned char i=0;i<(p/8)-i_base;i++){
    unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-i)));
    uint64_t bits=arr_geti(big_buffer,ints_addr);
    accu = loop_once(accu, cond_bits_c, bits);
    ints_addr++;
  }
  
  unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-((p/8)-i_base))));
  cond_bits_c &= (int)(0xFF)<<(8 - (p&0b111));
  uint64_t bits=arr_geti(big_buffer,ints_addr);
  accu = loop_once(accu, cond_bits_c, bits);
  
  result0 = result0^finalize(accu); 
  arr_set7c(little_buffer, k, result0);
}

#else
//#define __AVX2__
#if defined(__AVX2__) && (!defined (FIBO_NO_AVX))
//fast AVX implem
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

__attribute__((always_inline)) inline accumulator zero_acc(void);
__attribute__((always_inline)) inline uint64_t finalize(accumulator acc);
__attribute__((always_inline)) inline void arr_set31c(__m256i value,ptrdiff_t base_index);

__attribute__((always_inline)) inline
accumulator zero_acc(){
  return (accumulator){_mm256_setzero_si256(),_mm256_setzero_si256(),
  _mm256_setzero_si256(),_mm256_setzero_si256(),
  _mm256_setzero_si256(),_mm256_setzero_si256(),
  _mm256_setzero_si256(),_mm256_setzero_si256()
};}

__attribute__((always_inline)) inline
void arr_set31c(__m256i value,ptrdiff_t base_index){
  arr_seti(little_buffer,base_index,_mm256_extract_epi64(value, 0));
  arr_seti(little_buffer,base_index+8,_mm256_extract_epi64(value, 1));
  arr_seti(little_buffer,base_index+16,_mm256_extract_epi64(value, 2));
  arr_set7c(little_buffer, base_index+24, _mm256_extract_epi64(value, 3));
}


static char cond_mask[32];
static char p_mask[32];

__attribute__((always_inline)) inline

#define finalize1(j) 

uint64_t finalize(accumulator acc){
  __m256i temp;
  
  temp = _mm256_slli_epi64(acc.part1,64-1);
  acc.part1 = _mm256_srli_epi64(acc.part1,1);
  acc.part1 = _mm256_xor_si256(acc.part1,temp);

  
  //Due to an inversion during condition mask expending,
  //part0 contain, in this order, ints to be shifted of 3,2,1 and 0 bit,
  //while part1 contain the 7,6,5,4 ones 
  acc.part1=_mm256_srli_epi64 (acc.part1, 4); //now 3,2,1,0 as 4 done
  acc.part0=_mm256_xor_si256 (acc.part0, acc.part1); //xor-ing together as same shift to be donne
  __m128i second_half = _mm256_extracti128_si256 (acc.part0, 1); //contain 1,0
  __m128i first_half = _mm256_castsi256_si128(acc.part0);        //contain 3,2
  first_half = _mm_srli_epi64(first_half,2);                     //now 1,0
  first_half = _mm_xor_si128(first_half, second_half);           //xoring together, contain 1,0
      
  uint64_t result = _mm_extract_epi64 (first_half, 0)>>1;      //extracting 1, so sift it
  result ^= _mm_extract_epi64 (first_half, 1);                 //xoring the two 0 together
  return result;
}

void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,uint64_t result0){
  char boundary_align=ints_addr%4; //to perform optimized 32-bit aligned memory read
  ints_addr-=boundary_align;
  boundary_align*=8;
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();
  uint64_t int_1=arr_geti(big_buffer,ints_addr)>>boundary_align;
  __m256i cond_m = _mm256_load_si256((__m256i*)cond_mask);
  
  
  for (;i_base<p/32;i_base++){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-4)>>(bit_addr_shift);
    __m256i cond_reg = _mm256_set_epi8(cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),
                  cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),
                  cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),
                  cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits);
    cond_reg = _mm256_and_si256(cond_reg,cond_m);

     cond_reg = _mm256_cmpeq_epi8(cond_reg,accu.zeros);
    __m128i cond_reg1 = _mm256_extracti128_si256 (cond_reg, 1);
    __m128i cond_reg2 = _mm256_castsi256_si128 (cond_reg);


    uint64_t int_2=arr_geti(big_buffer,ints_addr+8);
    if (boundary_align!=0)
      int_1 |= int_2<<(64-boundary_align);
    int_2 >>= boundary_align;
    __m128i int_reg=_mm_set_epi64x(int_2, int_1);
  
    cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR1);
    __m256i cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
    __m256i temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part0 = _mm256_xor_si256(accu.part0, temp);

    cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part1 = _mm256_xor_si256(accu.part1, temp);

    int_reg = _mm_srli_si128(int_reg,1);
    cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);  
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part0 = _mm256_xor_si256(accu.part0, temp);
  
    cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part1 = _mm256_xor_si256(accu.part1, temp);

    int_reg = _mm_srli_si128(int_reg,1);
    cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR1);  //rotate hi <- lo (lowest<-highest)
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part0 = _mm256_xor_si256(accu.part0, temp);

    cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part1 = _mm256_xor_si256(accu.part1, temp);

    int_reg = _mm_srli_si128(int_reg,1);
    cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)    
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part0 = _mm256_xor_si256(accu.part0, temp);

    cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)    
    cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
    temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
    temp = _mm256_andnot_si256 (cond_reg64, temp);
    accu.part1 = _mm256_xor_si256(accu.part1, temp);
        

    bit_addr-=4;
    ints_addr+=4;
    int_1=(int_1>>32) | (int_2<<32);
  }


  uint64_t cond_bits = arr_geti(big_buffer,bit_addr-4)>>(bit_addr_shift);
  __m256i cond_reg = _mm256_set_epi8(cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),cond_bits>>(3*8),
                cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),cond_bits>>(2*8),
                cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),cond_bits>>(1*8),
                cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits,cond_bits);
  cond_reg = _mm256_and_si256(cond_reg,cond_m);

  __m256i p_m_reg = _mm256_load_si256((__m256i*)p_mask);
  cond_reg = _mm256_and_si256(p_m_reg,cond_reg);
  
  cond_reg = _mm256_cmpeq_epi8(cond_reg,accu.zeros);
  __m128i cond_reg1 = _mm256_extracti128_si256 (cond_reg, 1);
  __m128i cond_reg2 = _mm256_castsi256_si128 (cond_reg);


  uint64_t int_2=arr_geti(big_buffer,ints_addr+8);
  if (boundary_align!=0)
    int_1 |= int_2<<(64-boundary_align);
  int_2 >>= boundary_align;
  __m128i int_reg=_mm_set_epi64x(int_2, int_1);
  
  cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR1);
  __m256i cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
  __m256i temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part0 = _mm256_xor_si256(accu.part0, temp);

  cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part1 = _mm256_xor_si256(accu.part1, temp);

  int_reg = _mm_srli_si128(int_reg,1);
  cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);  
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part0 = _mm256_xor_si256(accu.part0, temp);
  
  cond_reg1 = _mm_shuffle_epi32(cond_reg1, ROTATOR2);
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg1);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part1 = _mm256_xor_si256(accu.part1, temp);

  int_reg = _mm_srli_si128(int_reg,1);
  cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR1);  //rotate hi <- lo (lowest<-highest)
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part0 = _mm256_xor_si256(accu.part0, temp);

  cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part1 = _mm256_xor_si256(accu.part1, temp);

  int_reg = _mm_srli_si128(int_reg,1);
  cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)    
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part0 = _mm256_xor_si256(accu.part0, temp);

  cond_reg2 = _mm_shuffle_epi32(cond_reg2, ROTATOR2);  //rotate hi <- lo (lowest<-highest)    
  cond_reg64 = _mm256_cvtepi8_epi64(cond_reg2);
  temp = _mm256_permute4x64_epi64(_mm256_castsi128_si256(int_reg), 0);
  temp = _mm256_andnot_si256 (cond_reg64, temp);
  accu.part1 = _mm256_xor_si256(accu.part1, temp);
  
  
  result0 = result0^finalize(accu); 
  arr_set7c(little_buffer, k, result0);
}



#else 

//slowest 64bits only implem
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

__attribute__((always_inline)) inline accumulator zero_acc(void);
__attribute__((always_inline)) inline accumulator loop_once(accumulator acc,char condition, uint64_t bits);
__attribute__((always_inline)) inline uint64_t finalize(accumulator acc);

__attribute__((always_inline)) inline
accumulator zero_acc(){  return (accumulator){0, 0, 0, 0, 0, 0, 0, 0};}

#define LOOP_INNER(j) if ((condition&1<<(7-j))) {acc.part##j^=bits;}
__attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,char condition, uint64_t bits){
  LOOP_INNER(0)
  LOOP_INNER(1)
  LOOP_INNER(2)
  LOOP_INNER(3)
  LOOP_INNER(4)
  LOOP_INNER(5)
  LOOP_INNER(6)
  LOOP_INNER(7)
  return acc;
}

__attribute__((always_inline)) inline
uint64_t finalize(accumulator acc){
  return acc.part0^(acc.part1>>1)^(acc.part2>>2)^(acc.part3>>3)^(acc.part4>>4)^(acc.part5>>5)
    ^(acc.part6>>6)^(acc.part7>>7); 
}


void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,uint64_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  
  for (;i_base<=((ptrdiff_t)(p/8))-7;i_base+=7){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift);
  
    for (signed char i=6;i>=0;i--){
      unsigned char cond_bits_c =  (char)(cond_bits>>(8*i));
      uint64_t bits=arr_geti(big_buffer,ints_addr);
      accu = loop_once(accu, cond_bits_c, bits);      
      ints_addr++;
    }
    bit_addr-=7;
  }

  uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift);
  
  for (unsigned char i=0;i<(p/8)-i_base;i++){
    unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-i)));
    uint64_t bits=arr_geti(big_buffer,ints_addr);
    accu = loop_once(accu, cond_bits_c, bits);
    ints_addr++;
  }
  
  unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-((p/8)-i_base))));
  cond_bits_c &= 0xFF<<(8 - (p&0b111));
  uint64_t bits=arr_geti(big_buffer,ints_addr);
  accu = loop_once(accu, cond_bits_c, bits);
  
  result0 = result0^finalize(accu); 
  arr_set7c(little_buffer, k, result0);
}

#endif
#endif


void jump_formula_plus1(void* k_arg){
  size_t k=(size_t)k_arg;

  uint64_t result=0;
  size_t ints_addr;
  ptrdiff_t bit_addr;
  
  if (k==0){
    //n=-1, n_p = 0

    if (arr_getb(big_buffer, 0)) { //0 bc that is the value of n_p
        result = arr_geti(big_buffer, 0);
        result =  result<<1 | ((result&1) ^ arr_getb(big_buffer, p) );
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p+1; //p+n_p  +1 for index
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1...
    size_t n_p = ((k+1)/2)*8;

    if (arr_getb(big_buffer, n_p))
      result = arr_geti(big_buffer,n/8)>>(n%8);
    
    ints_addr=(n+1)/8;
    bit_addr=n_p+p+1; //plus 1 because passing from 0 based indexing to 1 based (internal of jump_formulae_internal)
  }
  jump_formula_internal(k,ints_addr , bit_addr/8,bit_addr%8,result);
}

/* Applique la formule du jump, et calcule la range Fp(2n-8k **+0** ) à Fp(2n-8(k+7)+1 +0 ),placés dans little buffer, etant donné big_buffer rempli de suffiseament de Fp(n) et indices inférieurs
*  Valid if and only if p>=7
*/


void jump_formula(void* k_arg){
  size_t k=(size_t)k_arg;

  uint64_t result=0;
  size_t ints_addr;
  ptrdiff_t bit_addr;
  
  if (k==0){
    //to be easier in int reading,n must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    //n=-1, n_p = 1 
    if (arr_getb(big_buffer, 1)) { //1 bc that is the value of n_p
        result = arr_geti(big_buffer, 0);
        result =  result<<1 | ((result&1) ^ arr_getb(big_buffer, p) );
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p+1+1; //p+n_p  +1 for index
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    size_t n_p = ((k+1)/2)*8 +1 ;
  
    if (arr_getb(big_buffer, n_p))
      result = arr_geti(big_buffer,n/8)>>(n%8);
    
    ints_addr=(n+1)/8;
    bit_addr=n_p+p+1; //plus 1 because passing from 0 based indexing to 1 based (internal of jump_formulae_internal)
  }
  jump_formula_internal(k,ints_addr , bit_addr/8,bit_addr%8,result);
  
}

//assume init_max<=p+1 and init_max<=last_valid<=big_buffer_size*8
void initialize_big(size_t last_valid,size_t init_max){
  init_max+=1;
  for (size_t i=0;i<init_max>>3;i++){
    arr_setc(big_buffer,i,0xFF);
  }
  arr_setc(big_buffer,init_max>>3,0xFF>>(8-(init_max&0b111)));
  for (size_t i=(init_max>>3) +1;i<little_buffer_size;i++){
    arr_setc(big_buffer,i,0);
  }
  for (size_t i=0;i<last_valid-(p+1);i++){
    //TODO can be optimized, especially for big p
    arr_setb(big_buffer,i+p+1, arr_getb(big_buffer,i) ^ arr_getb(big_buffer, i+1) );
  }
  
}

//assuming that the last p+1 bits of little buffer are valid, refill big_buffer wiht last_valid valid bits at least
//assume p+1 < last_valid < 8*big_buffer_size
void refill_big_from_little(size_t last_valid){
  for (size_t i=0; i<((p+1)>>3)+1; i+=8) {
    arr_seti(big_buffer,i,arr_geti(little_buffer,i));
  }

  for (size_t i=0;i<last_valid-(p+1);i++){
    //TODO can be optimized, especially for big p
    arr_setb(big_buffer,i+p+1, arr_getb(big_buffer,i) ^ arr_getb(big_buffer, i+1) );
  }
}

//shift the p+1 last values of litle_buffer
void add_one(){
  bool last_bit=arr_getb(little_buffer,p+1) ^ arr_getb2(little_buffer, 0,0);
  for (size_t i=0;i<little_buffer_size;i+=7){
    uint64_t temp=arr_geti(little_buffer, i);
    temp=(temp<<1) + last_bit;
    last_bit = temp >> (8*7);
    arr_set7c(little_buffer, i, temp);
  }
}

size_t mpz_get_siz(mpz_t z)
{
      size_t ret;
      const size_t wordSize = sizeof(size_t);
      size_t wordCount = 0;
      size_t* outRaw = mpz_export(NULL, &wordCount, 1, wordSize, 0, 0, z);
      if (wordCount==0) return 0;
      ret = outRaw[0];
      void (*freeFunction)(void*, size_t);
      mp_get_memory_functions(NULL,NULL,&freeFunction);
      freeFunction(outRaw, wordCount * wordSize);
      return ret;
}

unsigned char* rust_fibo_mod2(size_t p_arg, const char* n);

unsigned char* rust_fibo_mod2(size_t p_arg, const char* n) {
    mpz_t n_mpz;
    mpz_init(n_mpz);
    mpz_set_str(n_mpz, n, 10);
    unsigned char* result = fibo_mod2(p_arg, n_mpz);
    mpz_clear(n_mpz);
    return result;
}

unsigned char* fibo_mod2(size_t p_arg,mpz_t n){
  if (mpz_cmp_ui(n,0)<0){
    printf("negative integers not yet supported, abborting");
    return NULL;
  }
  size_t min_valid_size = 10*(MIN(2*p_arg+3,p_arg+36+p_arg/2)) ;
  if (p!=p_arg || little_buffer==NULL || big_buffer==NULL){

    array_free(big_buffer, big_buffer_size);
    array_free(little_buffer, little_buffer_size);

    
    p = p_arg;
    if (min_valid_size<p) {
      printf("OVERSIZED P: ABORTING");
      p=0;
      return NULL;
    }

    big_buffer_size    = ((min_valid_size+7)>>3) +8; //to be sure I dont break anything as i am careless with boundary ...
    big_buffer         = array_create(big_buffer_size);
    little_buffer_size = (p>>3) + 1;
    little_buffer      = array_create(little_buffer_size);

    if (big_buffer==NULL||little_buffer==NULL) {
      printf("NOT ENOUGH MEMORY: ABORTING");
      array_free(big_buffer, big_buffer_size);
      big_buffer=NULL;
      array_free(little_buffer, little_buffer_size);
      little_buffer=NULL;
      p=0;
      return NULL;
    }
    }  

  unsigned int bits_p = 0;
  for (size_t copy=p;copy!=0;copy >>= 1){
    bits_p++;
  }

  size_t bits_n = mpz_sizeinbase(n,2);
  if (bits_n<=63 && bits_n< (size_t)bits_p-1) {
    //we are just as fast by calculating them iteratively ...
    initialize_big(little_buffer_size*8,mpz_get_siz(n));
  return big_buffer;}
  //launch the big machine ...

  //specific AVX initialisation
  #ifdef __AVX512F__

  #elif defined (__AVX2__)
  __m256i temp = _mm256_set_epi8(-128,64,32,16,8,4,2,1,-128,64,32,16,8,4,2,1,-128,64,32,16,8,4,2,1,-128,64,32,16,8,4,2,1);
  _mm256_store_si256 ((__m256i *)cond_mask, temp);
  #define MASK_IF(value,index) (-(char)((value%32)>index))
  temp = _mm256_set_epi8(MASK_IF(p,0),MASK_IF(p,1),MASK_IF(p,2),MASK_IF(p,3),MASK_IF(p,4),MASK_IF(p,5),MASK_IF(p,6),MASK_IF(p,7),
    MASK_IF(p,8),MASK_IF(p,9),MASK_IF(p,10),MASK_IF(p,11),MASK_IF(p,12),MASK_IF(p,13),MASK_IF(p,14),MASK_IF(p,15),
    MASK_IF(p,16),MASK_IF(p,17),MASK_IF(p,18),MASK_IF(p,19),MASK_IF(p,20),MASK_IF(p,21),MASK_IF(p,22),MASK_IF(p,23),
    MASK_IF(p,24),MASK_IF(p,25),MASK_IF(p,26),MASK_IF(p,27),MASK_IF(p,28),MASK_IF(p,29),MASK_IF(p,30),MASK_IF(p,31));
   _mm256_store_si256 ((__m256i *)p_mask, temp);
  
  #endif
  

  //the point is to get to have work_buffer_2 filled up with value from n to n-p
  //to do that, we can: shift left (aka multiply by two) by using the jump formulae or shift left and add two (jump_plus2)
  //initialize to a somewhat big value by initial shifting

  //How it work: we examine n as a bitfield from left to right (most to less significant bit)
  //We take  some first bit, and shift to that value, then, for each remaining bits, we:
  //shift left
  //adding two if necessary (aka 1 on previous bit)
  //add one in the end if necessary
  size_t init=0;
  size_t index=MIN(64,bits_p-1);
  index=MIN(index,bits_n);

  void (*jump_function)(void*);
  
  for (int i= index-1;i>=0;i--){
    init+= ((size_t)(mpz_tstbit(n,bits_n-index+i)))<<i; 
  }
  index=bits_n-index-1;  
  
  initialize_big(min_valid_size, init);

  if (index==ULLONG_MAX)
    return big_buffer;
       
  while (index>=1) {

    if (mpz_tstbit(n,index))
      jump_function= (&jump_formula_plus1);
    else
      jump_function= (&jump_formula);
    
    for (size_t i=0;i<little_buffer_size;i+=7){
      thpool_add_work(calcul_pool, jump_function, (void*)i);
    }
    thpool_wait(calcul_pool);
    refill_big_from_little(min_valid_size);
    index--;
  }
  // handling by hand the last jump as we do not fill back the big buffer
  if (mpz_tstbit(n,0))
    jump_function= (&jump_formula_plus1);
  else
    jump_function= (&jump_formula);

  for (size_t i=0;i<little_buffer_size;i+=7){
    thpool_add_work(calcul_pool, jump_function, (void*)i);
  }
  thpool_wait(calcul_pool);
  return little_buffer;
}
