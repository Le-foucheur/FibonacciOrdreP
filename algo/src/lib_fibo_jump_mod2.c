#include "lib_fibo_jump_mod2.h"
#include "external/C-Thread-Pool/thpool.h"
#include <smmintrin.h>
#include <stdbool.h>
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
//do the actual heavy work
static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result);
//calculate the next range adding one (2n+1)
static void jump_formula_plus1(void* k);
//calculate the next range (2n)
static void jump_formula(void* k);
//calculate iteratively the previous terms needed for the formula
void refill_big_from_little(size_t last_valid);
//initialize big with initial value in range 0-n
static void initialize_big(size_t last_valid,ptrdiff_t init_max);
//mpz_t to size_t
size_t mpz_get_siz(mpz_t z);

static __attribute__((always_inline)) inline accumulator zero_acc(void);
static __attribute__((always_inline)) inline bytes_t finalize(accumulator acc, bytes_t result0);
static __attribute__((always_inline)) inline accumulator loop_once(accumulator acc,cond_t condition , bytes_t bits);


static unsigned char* big_buffer;
static size_t big_buffer_size;
unsigned char* little_buffer;
static size_t little_buffer_size;
static size_t p;

// get number of available calculation cores
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

//packed binary array helper function 
//index are always from the end ... aka index i if fibo(n-i,p)

unsigned char* arr_get_false_addr(unsigned char* real_addr,size_t size){    return real_addr+8+(size-1)*INDEX_FLAT;}

unsigned char* arr_get_real_addr(unsigned char* array, size_t size){  return array-8-(size+1)*INDEX_FLAT;}

unsigned char* array_create(size_t size){  
  unsigned char* array = calloc(size+15,1);
  if (array==NULL) {
    return NULL;
  }
  return arr_get_false_addr(array,size);}
void array_free(unsigned char* array,size_t size){ if (array==NULL) return;  free(arr_get_real_addr(array,size));}
unsigned char* array_realoc(unsigned char* array,size_t old_size,size_t new_size){ return arr_get_false_addr(realloc(arr_get_real_addr(array,old_size), new_size),new_size);}


unsigned char arr_getc(unsigned char* array,ptrdiff_t index){  return array[index*INDEX_MULT];}

uint64_t arr_geti(unsigned char* array,ptrdiff_t index){  return * ((uint64_t*)(array+(index*INDEX_MULT)-(7*INDEX_FLAT)));}

void arr_setc(unsigned char* array,ptrdiff_t index,unsigned char set){  array[index*INDEX_MULT]=set;}

void arr_seti(unsigned char* array,ptrdiff_t index,uint64_t set){  *(uint64_t*)(array+index*INDEX_MULT-7*INDEX_FLAT)=set;}

void arr_set7c(unsigned char* array,size_t index,uint64_t set){
  for (unsigned char i=0;i<7;i++){
    arr_setc(array,index+i,(unsigned char)((set>>(8*i))&0xFF));
  }
}

bool char_getb(unsigned char ch,unsigned char index){  return (bool)((ch>>index)&1);}
unsigned char char_setb(unsigned char ch,unsigned char index,bool set){  return (ch & MASK8(index)) | (set<<index);}

void arr_setb(unsigned char* array,ptrdiff_t index,bool set){
  arr_setc(array,index>>3,char_setb(arr_getc(array,index>>3), index&0b111, set));
}

uint64_t int_setb(uint64_t it,unsigned char index,bool set){ return (it & MASK64(index)) | set<<index;}
bool int_getb(uint64_t it,unsigned char index){ return (bool)((it>>index)&1);}

bool arr_getb2(unsigned char* array,size_t arr_index,unsigned char c_index){  return char_getb(arr_getc(array,arr_index), c_index);}
bool arr_getb(unsigned char* array,size_t index){return arr_getb2(array, index>>3, (unsigned char)(index&0b111));}


#if FIBO_IMPLEM == 'T'
//******************************* AVX512 new test *************************************************
__m512i arr_get8i(unsigned char* array,ptrdiff_t index){  return _mm512_loadu_si512((__m512i*)(array+(index*INDEX_MULT)-(INDEX_FLAT*(8*8-1)))) ;}

__attribute__((always_inline)) inline
void arr_set63c(unsigned char* array,ptrdiff_t base_index,__m512i value){
  _mm512_mask_storeu_epi8(array+base_index*INDEX_MULT-(31*INDEX_FLAT),0x7FFFFFFFFFFFFFFFUL,value);
}

static accumulator zero_acc() {return (accumulator){_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),
                                             _mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32() };}

static __attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,cond_t condition, bytes_t bits){
  cond_t temp = _kshiftri_mask16(condition,1);
  acc.part0 = _mm512_mask_xor_epi64 (acc.part0, condition, acc.part0, bits);
  acc.part1 = _mm512_mask_xor_epi64 (acc.part1, temp, acc.part1, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part2 = _mm512_mask_xor_epi64 (acc.part2, condition, acc.part2, bits);
  acc.part3 = _mm512_mask_xor_epi64 (acc.part3, temp, acc.part3, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part4 = _mm512_mask_xor_epi64 (acc.part4, condition, acc.part4, bits);
  acc.part5 = _mm512_mask_xor_epi64 (acc.part5, temp, acc.part5, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part6 = _mm512_mask_xor_epi64 (acc.part6, condition, acc.part6, bits);
  acc.part7 = _mm512_mask_xor_epi64 (acc.part7, temp, acc.part7, bits);
  
  return acc;
}

static __attribute__((always_inline)) inline
bytes_t finalize(accumulator acc, bytes_t result0){
  /* How much to shift (right)?
     integer lane  
   0|1|2|3|4|5|6|7 
  
a0 7|6|5|4|3|2|1|0  
c1 6|5|4|3|2|1|0|7
c2 5|4|3|2|1|0|7|6
.3 4|3|2|1|0|7|6|5
p4 3|2|1|0|7|6|5|4
a5 2|1|0|7|6|5|4|3
r6 1|0|7|6|5|4|3|2
t7 0|7|6|5|4|3|2|1

r0 7|7|7|7|7|7|7|7
*/
  __m512i temp = _mm512_alignr_epi64(result0,result0,1);
  __m512i temp2;
  result0 = _mm512_shrdi_epi64 (result0, temp, 7);
  __m512i shifter = _mm512_set_epi64(0,1,2,3,4,5,6,7);
  __m512i less_one = _mm512_set1_epi64(-1);
  __m512i seven = _mm512_set1_epi64(7);

  __m512i next_shifter=_mm512_add_epi64(shifter,less_one);
  //TODO: mix temp instruction to hide latency from alignr
  temp = _mm512_alignr_epi64(acc.part7,acc.part7,1);      //bit that will be lost by right shifting should in fact go to the next lane, so we get in temp
  temp2 = _mm512_alignr_epi64(acc.part0,acc.part0,1);         //taking it now as we are on the verge of editing part0
  acc.part0 = _mm512_shrdv_epi64(acc.part0,temp,shifter); //bits from next lanes the should be sfifted of same amount (aka, previous accumulator) and then use shrdv
                                                          //to shift them both in one instruction
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part0);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part1,acc.part1,1);
  acc.part1 = _mm512_shrdv_epi64(acc.part1,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part1);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part2,acc.part2,1);
  acc.part2 = _mm512_shrdv_epi64(acc.part2,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part2);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part3,acc.part3,1);
  acc.part3 = _mm512_shrdv_epi64(acc.part3,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part3);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part4,acc.part4,1);
  acc.part4 = _mm512_shrdv_epi64(acc.part4,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part4);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part5,acc.part5,1);
  acc.part5 = _mm512_shrdv_epi64(acc.part5,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part5);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part6,acc.part6,1);
  acc.part6 = _mm512_shrdv_epi64(acc.part6,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part6);
  acc.part7 = _mm512_shrdv_epi64(acc.part7,temp2,shifter);  
  result0 = _mm512_xor_epi64(result0,acc.part7);
  
  return result0;
}
#define MASK_0246 0x00FF00FF00FF00FFUL
#define MASK_135  0x0000FF00FF00FF00UL
#define PACKER    0x101


static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  //the same loop is executed p/8 + 1 times, however condition have memory access economies by getting them by int batchs, so we
  //exute the loop by batches of 7
  
  for (;i_base<=(ptrdiff_t)(p)-56;i_base+=56){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
    uint64_t part1 = cond_bits & MASK_135;
    cond_bits &= MASK_0246;
    part1 *= PACKER;
    cond_bits*= PACKER;
    
    bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(6*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(5*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(4*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(3*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(2*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(1*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(0*8), int_bits);
    ints_addr++;
    
    
    bit_addr-=7;
  }

    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
    cond_bits &= 0xFFFFFFFFFFFFFFFFUL<<(56-(p-i_base));
    uint64_t part1 = cond_bits & MASK_135;
    cond_bits &= MASK_0246;
    part1 *= PACKER;
    cond_bits*= PACKER;
    
    bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(6*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(5*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(4*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(3*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(2*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(1*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(0*8), int_bits);
    ints_addr++;



  result0 = finalize(accu,result0);         //compact the values in the accumulator and initial value
  arr_set_result(little_buffer, k, result0);//write to memory
}

#endif
#if FIBO_IMPLEM == '5'
//******************************* fastest AVX-512 implem *******************************************

__m512i arr_get8i(unsigned char* array,ptrdiff_t index){  return _mm512_loadu_si512((__m512i*)(array+(index*INDEX_MULT)-(INDEX_FLAT*(8*8-1)))) ;}

static __attribute__((always_inline)) inline
void arr_set63c(unsigned char* array,ptrdiff_t base_index,__m512i value){
  _mm512_mask_storeu_epi8(array+base_index*INDEX_MULT-(31*INDEX_FLAT),0x7FFFFFFFFFFFFFFFUL,value);
}

static accumulator zero_acc() {return (accumulator){_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),
                                             _mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32(),_mm512_setzero_epi32() };}

static __attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,cond_t condition, bytes_t bits){
  cond_t temp = _kshiftri_mask16(condition,1);
  acc.part0 = _mm512_mask_xor_epi64 (acc.part0, condition, acc.part0, bits);
  acc.part1 = _mm512_mask_xor_epi64 (acc.part1, temp, acc.part1, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part2 = _mm512_mask_xor_epi64 (acc.part2, condition, acc.part2, bits);
  acc.part3 = _mm512_mask_xor_epi64 (acc.part3, temp, acc.part3, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part4 = _mm512_mask_xor_epi64 (acc.part4, condition, acc.part4, bits);
  acc.part5 = _mm512_mask_xor_epi64 (acc.part5, temp, acc.part5, bits);
  condition = _kshiftri_mask16(condition, 2);
  temp = _kshiftri_mask16(temp,2);
  acc.part6 = _mm512_mask_xor_epi64 (acc.part6, condition, acc.part6, bits);
  acc.part7 = _mm512_mask_xor_epi64 (acc.part7, temp, acc.part7, bits);
  
  return acc;
}

static __attribute__((always_inline)) inline
bytes_t finalize(accumulator acc, bytes_t result0){
  /* How much to shift (right)?
     integer lane  
   0|1|2|3|4|5|6|7 
  
a0 7|6|5|4|3|2|1|0  
c1 6|5|4|3|2|1|0|7
c2 5|4|3|2|1|0|7|6
.3 4|3|2|1|0|7|6|5
p4 3|2|1|0|7|6|5|4
a5 2|1|0|7|6|5|4|3
r6 1|0|7|6|5|4|3|2
t7 0|7|6|5|4|3|2|1

r0 7|7|7|7|7|7|7|7
*/
  __m512i temp = _mm512_alignr_epi64(result0,result0,1);
  __m512i temp2;
  result0 = _mm512_shrdi_epi64 (result0, temp, 7);
  __m512i shifter = _mm512_set_epi64(0,1,2,3,4,5,6,7);
  __m512i less_one = _mm512_set1_epi64(-1);
  __m512i seven = _mm512_set1_epi64(7);

  __m512i next_shifter=_mm512_add_epi64(shifter,less_one);
  //TODO: mix temp instruction to hide latency from alignr
  temp = _mm512_alignr_epi64(acc.part7,acc.part7,1);      //bit that will be lost by right shifting should in fact go to the next lane, so we get in temp
  temp2 = _mm512_alignr_epi64(acc.part0,acc.part0,1);         //taking it now as we are on the verge of editing part0
  acc.part0 = _mm512_shrdv_epi64(acc.part0,temp,shifter); //bits from next lanes the should be sfifted of same amount (aka, previous accumulator) and then use shrdv
                                                          //to shift them both in one instruction
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part0);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part1,acc.part1,1);
  acc.part1 = _mm512_shrdv_epi64(acc.part1,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part1);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part2,acc.part2,1);
  acc.part2 = _mm512_shrdv_epi64(acc.part2,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part2);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part3,acc.part3,1);
  acc.part3 = _mm512_shrdv_epi64(acc.part3,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part3);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part4,acc.part4,1);
  acc.part4 = _mm512_shrdv_epi64(acc.part4,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part4);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp = _mm512_alignr_epi64(acc.part5,acc.part5,1);
  acc.part5 = _mm512_shrdv_epi64(acc.part5,temp2,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part5);
  next_shifter = _mm512_add_epi64(next_shifter,less_one);
  temp2 = _mm512_alignr_epi64(acc.part6,acc.part6,1);
  acc.part6 = _mm512_shrdv_epi64(acc.part6,temp,shifter);
  
  shifter = _mm512_and_epi64(next_shifter,seven);
  result0 = _mm512_xor_epi64(result0,acc.part6);
  acc.part7 = _mm512_shrdv_epi64(acc.part7,temp2,shifter);  
  result0 = _mm512_xor_epi64(result0,acc.part7);
  
  return result0;
}
#define MASK_0246 0x00FF00FF00FF00FFUL
#define MASK_135  0x0000FF00FF00FF00UL
#define PACKER    0x101


static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  //the same loop is executed p/8 + 1 times, however condition have memory access economies by getting them by int batchs, so we
  //exute the loop by batches of 7
  
  for (;i_base<=(ptrdiff_t)(p)-56;i_base+=56){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
    uint64_t part1 = cond_bits & MASK_135;
    cond_bits &= MASK_0246;
    part1 *= PACKER;
    cond_bits*= PACKER;
    
    bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(6*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(5*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(4*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(3*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(2*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(1*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(0*8), int_bits);
    ints_addr++;
    
    
    bit_addr-=7;
  }

    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
    cond_bits &= 0xFFFFFFFFFFFFFFFFUL<<(56-(p-i_base));
    uint64_t part1 = cond_bits & MASK_135;
    cond_bits &= MASK_0246;
    part1 *= PACKER;
    cond_bits*= PACKER;
    
    bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(6*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(5*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(4*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(3*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(2*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, part1>>(1*8), int_bits);
    ints_addr++;
    
    int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
    accu = loop_once(accu, cond_bits>>(0*8), int_bits);
    ints_addr++;



  result0 = finalize(accu,result0);         //compact the values in the accumulator and initial value
  arr_set_result(little_buffer, k, result0);//write to memory
}


#endif
#if FIBO_IMPLEM == '2'
//************************** fast? AVX implem ********************

#define mm256_blendv_epi64(A,B,M) \
  _mm256_castpd_si256(_mm256_blendv_pd(_mm256_castsi256_pd(A),_mm256_castsi256_pd(B),_mm256_castsi256_pd(M)))

static __attribute__((always_inline)) inline
accumulator zero_acc(){
  return (accumulator){_mm256_setzero_si256(),_mm256_setzero_si256(),_mm256_setzero_si256(),_mm256_setzero_si256(),
                      _mm256_setzero_si256(),_mm256_setzero_si256(),_mm256_setzero_si256(),_mm256_setzero_si256(),_mm256_setzero_si256()};}

__attribute__((always_inline)) inline
void arr_set31c(unsigned char* array,ptrdiff_t base_index,__m256i value){
  arr_seti(array,base_index,_mm256_extract_epi64(value, 0));
  arr_seti(array,base_index+8,_mm256_extract_epi64(value, 1));
  arr_seti(array,base_index+16,_mm256_extract_epi64(value, 2));
  arr_set7c(array, base_index+24, _mm256_extract_epi64(value, 3));
}


#define finalize1(j) temp = _mm256_slli_epi64(acc.part##j,64-j); \
  temp = _mm256_permute4x64_epi64(temp,0b00111001); \
  acc.part0 = _mm256_xor_si256(acc.part0,temp); \
  acc.part##j = _mm256_srli_epi64(acc.part##j,j); \
  acc.part0 = _mm256_xor_si256(acc.part0,acc.part##j);


static __attribute__((always_inline)) inline
bytes_t finalize(accumulator acc,bytes_t result0){
  acc.part7 = _mm256_xor_si256(acc.part7,result0);
  __m256i temp;
  
  finalize1(1)
  finalize1(2)
  finalize1(3)
  finalize1(4)
  finalize1(5)
  finalize1(6)
  finalize1(7)
  
  
  return acc.part0;
}

__m256i arr_get8i(unsigned char* array,ptrdiff_t index){  return _mm256_loadu_si256((__m256i*)(array+(index*INDEX_MULT)-(INDEX_FLAT*(8*4-1)))) ;}
__m256i arr_broadload(unsigned char* array,ptrdiff_t index){ return (__m256i)(_mm256_broadcast_sd((double*)(array+(index*INDEX_MULT)-(INDEX_FLAT*7))));}



/* #define LOOP_INNER(j) temp = _mm256_xor_si256(bits,acc.part##j); \
    acc.part##j = mm256_blendv_epi64 (acc.part##j, temp, acc.cond); \
    acc.cond = _mm256_slli_epi64 (acc.cond, 1); */


static __attribute__((always_inline)) inline
accumulator loop_once(accumulator acc, cond_t condition, bytes_t bits){
    __m256i temp;
    __m256i temp2;
    __m256i tempcond;

    //to try and hide the latency, we mix the intructions from two set of three in order to do shift/xor (other unrelated) blendv
  
    temp = _mm256_xor_si256(bits,acc.part0);                    // 0.1
    tempcond = _mm256_slli_epi64 (acc.cond, 1);                 //1.0
    temp2 = _mm256_xor_si256(bits,acc.part1);                   //1.1
    acc.part0 = mm256_blendv_epi64 (acc.part0, temp, acc.cond); // 0.2
    acc.part1 = mm256_blendv_epi64 (acc.part1, temp2, tempcond);//1.2
  
    acc.cond = _mm256_slli_epi64 (acc.cond, 2);                 //2.0
    temp = _mm256_xor_si256(bits,acc.part2);                    //2.1
    tempcond = _mm256_slli_epi64 (tempcond, 2);                 // 3.0
    temp2 = _mm256_xor_si256(bits,acc.part3);                   // 3.1
    acc.part2 = mm256_blendv_epi64 (acc.part2, temp, acc.cond); //2.2
    acc.part3 = mm256_blendv_epi64 (acc.part3, temp2, acc.cond);// 3.2
  
    acc.cond = _mm256_slli_epi64 (acc.cond, 2);                 //4.0
    temp = _mm256_xor_si256(bits,acc.part4);                    //4.1
    tempcond = _mm256_slli_epi64 (tempcond, 2);                 //5.0
    temp2 = _mm256_xor_si256(bits,acc.part5);                   //5.1
    acc.part4 = mm256_blendv_epi64 (acc.part4, temp, acc.cond); //4.2
    acc.part5 = mm256_blendv_epi64 (acc.part5, temp2, acc.cond);//5.2
  
    acc.cond = _mm256_slli_epi64 (acc.cond, 2);                 //6.0
    temp = _mm256_xor_si256(bits,acc.part6);                    //6.1
    tempcond = _mm256_slli_epi64 (tempcond, 2);                 //7.0
    temp2 = _mm256_xor_si256(bits,acc.part7);                   //7.1
    acc.part6 = mm256_blendv_epi64 (acc.part6, temp, acc.cond); //6.2
    acc.part7 = mm256_blendv_epi64 (acc.part7, temp2, acc.cond);//7.2
  
    acc.cond = _mm256_slli_epi64 (acc.cond, 2);                 //7.3
  
  
    //LOOP_INNER(0)    //LOOP_INNER(1)    //LOOP_INNER(2)    //LOOP_INNER(3)    //LOOP_INNER(4)    //LOOP_INNER(5)    //LOOP_INNER(6)    //LOOP_INNER(7)
  return acc;
}

static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  //the same loop is executed p/8 + 1 times, however condition have memory access economies by getting them by int batchs, so we
  //exute the loop by batches of 7
  
  for (;i_base<=(ptrdiff_t)(p)-56;i_base+=56){
    //uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)<<(8-bit_addr_shift); //get a pack of 56 condition
    //accu.cond = _mm256_set1_epi64x (cond_bits);
    accu.cond = arr_broadload(big_buffer, bit_addr-7);
    accu.cond = _mm256_slli_epi64(accu.cond, (8-bit_addr_shift));
    for (char i=0;i<7;i++){
      bytes_t bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes treated by the condition
      accu = loop_once(accu, (cond_t){}, bits);      //treat 8 condition packed in a char
      ints_addr++;
    }
    bit_addr-=7;
  }

  //uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get the last pack of condition, used for the remainings of the formula
  //accu.cond = _mm256_set1_epi64x (cond_bits);
  accu.cond = arr_broadload(big_buffer, bit_addr-7);
  //(8-bit_addr_shift) left would keep 56 valid bits. we want to keep p-i_base valid bits
  //so we shift right of 64-(p-i_base)-(8-bit_addr_shift)
  //the -(8-bit_addr_shift) would have left aligned the valid bytes, so we go left by 64
  //less how much we want to keep
  accu.cond = _mm256_srli_epi64(accu.cond, 64-(p-i_base)-(8-bit_addr_shift));
  //then we go back by th same amount, and shift left of (8-bit_addr_shift) to left-align
  //wich sum up to 64-(p-i_base)
  accu.cond = _mm256_slli_epi64(accu.cond,64-(p-i_base));

  
  for (;i_base<p;i_base+=8){                          //treat the part of the last 56 conditions wich are still packed by 8
      bytes_t bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes treated by the condition
      accu = loop_once(accu, (cond_t){}, bits);      //treat 8 condition packed in a char
      ints_addr++;
  }
  
  result0 = finalize(accu,result0);         //compact the values in the accumulator and initial value
  arr_set_result(little_buffer, k, result0);//write to memory
}
#endif

#if FIBO_IMPLEM == 'i'
//*********** slowest 64bits only implem ***********************

static __attribute__((always_inline)) inline
accumulator zero_acc(){  return (accumulator){0, 0, 0, 0, 0, 0, 0, 0};}

#define LOOP_INNER(j) if ((condition&1<<(7-j))) {acc.part##j^=bits;}
static __attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,cond_t condition, bytes_t bits){
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

static __attribute__((always_inline)) inline
bytes_t finalize(accumulator acc,bytes_t result0){
  return acc.part0^(acc.part1>>1)^(acc.part2>>2)^(acc.part3>>3)^(acc.part4>>4)^(acc.part5>>5)
    ^(acc.part6>>6)^((result0^acc.part7)>>7); 
}

static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  //the same loop is executed p/8 + 1 times, however condition have memory access economies by getting them by int batchs, so we
  //exute the loop by batches of 7
  
  for (;i_base<=((ptrdiff_t)(p/8))-7;i_base+=7){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
  
    for (signed char i=6;i>=0;i--){
      unsigned char cond_bits_c =  (char)(cond_bits>>(8*i));
      bytes_t bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes treated by the condition
      accu = loop_once(accu, cond_bits_c, bits);      //treat 8 condition packed in a char
      ints_addr++;
    }
    bit_addr-=7;
  }

  uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get the last pack of condition, used for the remainings of the formula
  
  for (unsigned char i=0;i<(p/8)-i_base;i++){                             //treat the part of the last 56 conditions wich are still packed by 8
    unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-i)));
    bytes_t bits=get_bytes(big_buffer,ints_addr);
    accu = loop_once(accu, cond_bits_c, bits);
    ints_addr++;
  }
  
  unsigned char cond_bits_c =  (char)(cond_bits>>(8*(6-((p/8)-i_base)))); //mask the remainning last few condition
  cond_bits_c &= (int)(0xFF)<<(8 - (p&0b111));
  bytes_t bits=get_bytes(big_buffer,ints_addr);
  accu = loop_once(accu, cond_bits_c, bits);
  
  result0 = finalize(accu,result0);         //compact the values in the accumulator and initial value
  arr_set_result(little_buffer, k, result0);//write to memory
}

#endif
/*************** END SPECIFIC IMPLEMENTATIONS *******************/

void jump_formula_plus1(void* k_arg){
  size_t k=(size_t)k_arg;

  bytes_t result= byte_zero;
  size_t ints_addr;
  ptrdiff_t bit_addr;
  
  if (k==0){
    //n=-1, n_p = 0

    if (arr_getb(big_buffer, 0)) { //0 bc that is the value of n_p
        //we know we have some margin: lets use it (i know thats ugly ... but anyway)
        arr_setb(big_buffer,-1,(arr_getb(big_buffer, 0) ^ arr_getb(big_buffer, p) ));
        result = get_bytes(big_buffer, -1); //jump formulae internal know it should be right shifted of 7
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p+1; //p+n_p  +1 for index
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1...
    size_t n_p = ((k+1)/2)*8;

    if (arr_getb(big_buffer, n_p))
      result = get_bytes(big_buffer,n/8);
    
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

  bytes_t result = byte_zero;
  size_t ints_addr;
  ptrdiff_t bit_addr;
  
  if (k==0){
    //to be easier in int reading,n must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    //n=-1, n_p = 1 
    if (arr_getb(big_buffer, 1)) { //1 bc that is the value of n_p
        //we know we have some margin: lets use it (i know thats ugly ... but anyway)
        arr_setb(big_buffer,-1,(arr_getb(big_buffer, 0) ^ arr_getb(big_buffer, p) ));
        result = get_bytes(big_buffer, -1); //jump formulae internal know it should be right shifted of 7
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p+1+1; //p+n_p  +1 for index
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    size_t n_p = ((k+1)/2)*8 +1 ;
  
    if (arr_getb(big_buffer, n_p))
      result = get_bytes(big_buffer,n/8);
    
    ints_addr=(n+1)/8;
    bit_addr=n_p+p+1; //plus 1 because passing from 0 based indexing to 1 based (internal of jump_formulae_internal)
  }
  jump_formula_internal(k,ints_addr , bit_addr/8,bit_addr%8,result);
  
}

//assume init_max<=p+1 and init_max<=last_valid<=big_buffer_size*8
void initialize_big(size_t last_valid,ptrdiff_t init_max){
  if (init_max>=0){
    init_max+=1; //now, init_max = number of 1
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
  } else {
    for (ptrdiff_t i=-1;i<(ptrdiff_t)(last_valid>>3)+1;i++)
      arr_setc(big_buffer, i, 0);
    arr_setb(big_buffer,p+init_max+1,1);
    for (ptrdiff_t i=p+init_max;i<last_valid-(p+1);i++){
      //TODO can be optimized, especially for big p
      arr_setb(big_buffer,i+p+1, arr_getb(big_buffer,i) ^ arr_getb(big_buffer, i+1) );
    }
    
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

unsigned char* fibo_mod2(size_t p_arg,mpz_t n){
  size_t min_valid_size = (MIN(2*p_arg+4,p_arg+p_arg/2+7*BATCH_SIZE*8+4)) ;
  p = p_arg;

  unsigned int bits_p = 0;
  for (size_t copy=p;copy!=0;copy >>= 1){
    bits_p++;
  }
  bool neg_n = mpz_cmp_ui(n,0)<0;
  mpz_abs(n,n);
  
  size_t bits_n = mpz_sizeinbase(n,2);
  if ((bits_n<=63 && bits_n< (size_t)bits_p-1) || (p==1 && bits_n==1)) {
    //we are just as fast by calculating them iteratively ...
    if (neg_n)
      initialize_big(little_buffer_size*8, -(ptrdiff_t)(mpz_get_siz(n)));
    else
      initialize_big(little_buffer_size*8,mpz_get_siz(n));
    return big_buffer;
  }
  //launch the big machine ...
  

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
  index=MIN(index,bits_n-1);
  void (*jump_function_1)(void*);
  void (*jump_function_0)(void*);
  
  if (neg_n){
    mpz_sub_ui(n,n,1);
    jump_function_0 = &jump_formula_plus1;
    jump_function_1 = &jump_formula;
  } else {
    jump_function_0 = &jump_formula;
    jump_function_1 = &jump_formula_plus1;
  }

  void (*jump_function)(void*);
  
  for (int i= index-1;i>=0;i--){
    init+= ((size_t)(mpz_tstbit(n,bits_n-index+i)))<<i; 
  }
  index=bits_n-index-1;  
  
  initialize_big(min_valid_size, neg_n ? -(init +neg_n):init);

  if (index==ULLONG_MAX)
    return big_buffer;
       
  while (index>=1) {

    if (mpz_tstbit(n,index))
      jump_function= jump_function_1;
    else
      jump_function= jump_function_0;
    
    for (size_t i=0;i<little_buffer_size;i+=BATCH_SIZE){
      thpool_add_work(calcul_pool, jump_function, (void*)i);
    }
    thpool_wait(calcul_pool);
    refill_big_from_little(min_valid_size);
    index--;
  }
  // handling by hand the last jump as we do not fill back the big buffer
  if (mpz_tstbit(n,0))
    jump_function= jump_function_1;
  else
    jump_function= jump_function_0;

  for (size_t i=0;i<little_buffer_size;i+=BATCH_SIZE){
    thpool_add_work(calcul_pool, jump_function, (void*)i);
  }
  thpool_wait(calcul_pool);
  return little_buffer;
}


// Init functions to call malloc one time for a serie of p
// return 1 on error
int fibo_mod2_initialization(size_t p_arg){
  size_t min_valid_size = (MIN(2*p_arg+4,p_arg+p_arg/2+7*BATCH_SIZE*8+4)) ;
  array_free(big_buffer, big_buffer_size);
  array_free(little_buffer, little_buffer_size);

  if (min_valid_size<p_arg) {
    printf("OVERSIZED P: ABORTING");
    big_buffer=NULL;
    little_buffer=NULL;
    return 1;
  }

  big_buffer_size    = ((min_valid_size+7)>>3) +BATCH_SIZE + 8; //to be sure I dont break anything as i am careless with boundary ...
  big_buffer         = array_create(big_buffer_size);
  little_buffer_size = (p_arg>>3) + 1;
  little_buffer      = array_create(little_buffer_size + BATCH_SIZE + 8);

  if (big_buffer==NULL||little_buffer==NULL) {
    printf("NOT ENOUGH MEMORY: ABORTING");
    array_free(big_buffer, big_buffer_size);
    big_buffer=NULL;
    array_free(little_buffer, little_buffer_size);
    little_buffer=NULL;
    return 1;
  }
  return 0;
}
