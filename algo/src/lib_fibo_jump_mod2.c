#include "lib_fibo_jump_mod2.h"
#include "external/C-Thread-Pool/thpool.h"
#include "external/gmp-6.3.0/gmp.h"
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

#define LOOP_I(j) acc.part##j = _mm512_xor_epi64 (acc.part##j,bits)
#define LOOP_8(mask) \
  if (mask&128)LOOP_I(0); \
  if (mask&64) LOOP_I(1); \
  if (mask&32) LOOP_I(2); \
  if (mask&16) LOOP_I(3); \
  if (mask&8)  LOOP_I(4);\
  if (mask&4)  LOOP_I(5);\
  if (mask&2)  LOOP_I(6);\
  if (mask&1)  LOOP_I(7);


static __attribute__((always_inline)) inline
accumulator loop_once(accumulator acc,cond_t condition, bytes_t bits){
  switch (condition) {
    case 0: LOOP_8(0) break;
    case 1: LOOP_8(1) break;
    case 2: LOOP_8(2) break;
    case 3: LOOP_8(3) break;
    case 4: LOOP_8(4) break;
    case 5: LOOP_8(5) break;
    case 6: LOOP_8(6) break;
    case 7: LOOP_8(7) break;
    case 8: LOOP_8(8) break;
    case 9: LOOP_8(9) break;
    case 10: LOOP_8(10) break;
    case 11: LOOP_8(11) break;
    case 12: LOOP_8(12) break;
    case 13: LOOP_8(13) break;
    case 14: LOOP_8(14) break;
    case 15: LOOP_8(15) break;
    case 16: LOOP_8(16) break;
    case 17: LOOP_8(17) break;
    case 18: LOOP_8(18) break;
    case 19: LOOP_8(19) break;
    case 20: LOOP_8(20) break;
    case 21: LOOP_8(21) break;
    case 22: LOOP_8(22) break;
    case 23: LOOP_8(23) break;
    case 24: LOOP_8(24) break;
    case 25: LOOP_8(25) break;
    case 26: LOOP_8(26) break;
    case 27: LOOP_8(27) break;
    case 28: LOOP_8(28) break;
    case 29: LOOP_8(29) break;
    case 30: LOOP_8(30) break;
    case 31: LOOP_8(31) break;
    case 32: LOOP_8(32) break;
    case 33: LOOP_8(33) break;
    case 34: LOOP_8(34) break;
    case 35: LOOP_8(35) break;
    case 36: LOOP_8(36) break;
    case 37: LOOP_8(37) break;
    case 38: LOOP_8(38) break;
    case 39: LOOP_8(39) break;
    case 40: LOOP_8(40) break;
    case 41: LOOP_8(41) break;
    case 42: LOOP_8(42) break;
    case 43: LOOP_8(43) break;
    case 44: LOOP_8(44) break;
    case 45: LOOP_8(45) break;
    case 46: LOOP_8(46) break;
    case 47: LOOP_8(47) break;
    case 48: LOOP_8(48) break;
    case 49: LOOP_8(49) break;
    case 50: LOOP_8(50) break;
    case 51: LOOP_8(51) break;
    case 52: LOOP_8(52) break;
    case 53: LOOP_8(53) break;
    case 54: LOOP_8(54) break;
    case 55: LOOP_8(55) break;
    case 56: LOOP_8(56) break;
    case 57: LOOP_8(57) break;
    case 58: LOOP_8(58) break;
    case 59: LOOP_8(59) break;
    case 60: LOOP_8(60) break;
    case 61: LOOP_8(61) break;
    case 62: LOOP_8(62) break;
    case 63: LOOP_8(63) break;
    case 64: LOOP_8(64) break;
    case 65: LOOP_8(65) break;
    case 66: LOOP_8(66) break;
    case 67: LOOP_8(67) break;
    case 68: LOOP_8(68) break;
    case 69: LOOP_8(69) break;
    case 70: LOOP_8(70) break;
    case 71: LOOP_8(71) break;
    case 72: LOOP_8(72) break;
    case 73: LOOP_8(73) break;
    case 74: LOOP_8(74) break;
    case 75: LOOP_8(75) break;
    case 76: LOOP_8(76) break;
    case 77: LOOP_8(77) break;
    case 78: LOOP_8(78) break;
    case 79: LOOP_8(79) break;
    case 80: LOOP_8(80) break;
    case 81: LOOP_8(81) break;
    case 82: LOOP_8(82) break;
    case 83: LOOP_8(83) break;
    case 84: LOOP_8(84) break;
    case 85: LOOP_8(85) break;
    case 86: LOOP_8(86) break;
    case 87: LOOP_8(87) break;
    case 88: LOOP_8(88) break;
    case 89: LOOP_8(89) break;
    case 90: LOOP_8(90) break;
    case 91: LOOP_8(91) break;
    case 92: LOOP_8(92) break;
    case 93: LOOP_8(93) break;
    case 94: LOOP_8(94) break;
    case 95: LOOP_8(95) break;
    case 96: LOOP_8(96) break;
    case 97: LOOP_8(97) break;
    case 98: LOOP_8(98) break;
    case 99: LOOP_8(99) break;
    case 100: LOOP_8(100) break;
    case 101: LOOP_8(101) break;
    case 102: LOOP_8(102) break;
    case 103: LOOP_8(103) break;
    case 104: LOOP_8(104) break;
    case 105: LOOP_8(105) break;
    case 106: LOOP_8(106) break;
    case 107: LOOP_8(107) break;
    case 108: LOOP_8(108) break;
    case 109: LOOP_8(109) break;
    case 110: LOOP_8(110) break;
    case 111: LOOP_8(111) break;
    case 112: LOOP_8(112) break;
    case 113: LOOP_8(113) break;
    case 114: LOOP_8(114) break;
    case 115: LOOP_8(115) break;
    case 116: LOOP_8(116) break;
    case 117: LOOP_8(117) break;
    case 118: LOOP_8(118) break;
    case 119: LOOP_8(119) break;
    case 120: LOOP_8(120) break;
    case 121: LOOP_8(121) break;
    case 122: LOOP_8(122) break;
    case 123: LOOP_8(123) break;
    case 124: LOOP_8(124) break;
    case 125: LOOP_8(125) break;
    case 126: LOOP_8(126) break;
    case 127: LOOP_8(127) break;
    case 128: LOOP_8(128) break;
    case 129: LOOP_8(129) break;
    case 130: LOOP_8(130) break;
    case 131: LOOP_8(131) break;
    case 132: LOOP_8(132) break;
    case 133: LOOP_8(133) break;
    case 134: LOOP_8(134) break;
    case 135: LOOP_8(135) break;
    case 136: LOOP_8(136) break;
    case 137: LOOP_8(137) break;
    case 138: LOOP_8(138) break;
    case 139: LOOP_8(139) break;
    case 140: LOOP_8(140) break;
    case 141: LOOP_8(141) break;
    case 142: LOOP_8(142) break;
    case 143: LOOP_8(143) break;
    case 144: LOOP_8(144) break;
    case 145: LOOP_8(145) break;
    case 146: LOOP_8(146) break;
    case 147: LOOP_8(147) break;
    case 148: LOOP_8(148) break;
    case 149: LOOP_8(149) break;
    case 150: LOOP_8(150) break;
    case 151: LOOP_8(151) break;
    case 152: LOOP_8(152) break;
    case 153: LOOP_8(153) break;
    case 154: LOOP_8(154) break;
    case 155: LOOP_8(155) break;
    case 156: LOOP_8(156) break;
    case 157: LOOP_8(157) break;
    case 158: LOOP_8(158) break;
    case 159: LOOP_8(159) break;
    case 160: LOOP_8(160) break;
    case 161: LOOP_8(161) break;
    case 162: LOOP_8(162) break;
    case 163: LOOP_8(163) break;
    case 164: LOOP_8(164) break;
    case 165: LOOP_8(165) break;
    case 166: LOOP_8(166) break;
    case 167: LOOP_8(167) break;
    case 168: LOOP_8(168) break;
    case 169: LOOP_8(169) break;
    case 170: LOOP_8(170) break;
    case 171: LOOP_8(171) break;
    case 172: LOOP_8(172) break;
    case 173: LOOP_8(173) break;
    case 174: LOOP_8(174) break;
    case 175: LOOP_8(175) break;
    case 176: LOOP_8(176) break;
    case 177: LOOP_8(177) break;
    case 178: LOOP_8(178) break;
    case 179: LOOP_8(179) break;
    case 180: LOOP_8(180) break;
    case 181: LOOP_8(181) break;
    case 182: LOOP_8(182) break;
    case 183: LOOP_8(183) break;
    case 184: LOOP_8(184) break;
    case 185: LOOP_8(185) break;
    case 186: LOOP_8(186) break;
    case 187: LOOP_8(187) break;
    case 188: LOOP_8(188) break;
    case 189: LOOP_8(189) break;
    case 190: LOOP_8(190) break;
    case 191: LOOP_8(191) break;
    case 192: LOOP_8(192) break;
    case 193: LOOP_8(193) break;
    case 194: LOOP_8(194) break;
    case 195: LOOP_8(195) break;
    case 196: LOOP_8(196) break;
    case 197: LOOP_8(197) break;
    case 198: LOOP_8(198) break;
    case 199: LOOP_8(199) break;
    case 200: LOOP_8(200) break;
    case 201: LOOP_8(201) break;
    case 202: LOOP_8(202) break;
    case 203: LOOP_8(203) break;
    case 204: LOOP_8(204) break;
    case 205: LOOP_8(205) break;
    case 206: LOOP_8(206) break;
    case 207: LOOP_8(207) break;
    case 208: LOOP_8(208) break;
    case 209: LOOP_8(209) break;
    case 210: LOOP_8(210) break;
    case 211: LOOP_8(211) break;
    case 212: LOOP_8(212) break;
    case 213: LOOP_8(213) break;
    case 214: LOOP_8(214) break;
    case 215: LOOP_8(215) break;
    case 216: LOOP_8(216) break;
    case 217: LOOP_8(217) break;
    case 218: LOOP_8(218) break;
    case 219: LOOP_8(219) break;
    case 220: LOOP_8(220) break;
    case 221: LOOP_8(221) break;
    case 222: LOOP_8(222) break;
    case 223: LOOP_8(223) break;
    case 224: LOOP_8(224) break;
    case 225: LOOP_8(225) break;
    case 226: LOOP_8(226) break;
    case 227: LOOP_8(227) break;
    case 228: LOOP_8(228) break;
    case 229: LOOP_8(229) break;
    case 230: LOOP_8(230) break;
    case 231: LOOP_8(231) break;
    case 232: LOOP_8(232) break;
    case 233: LOOP_8(233) break;
    case 234: LOOP_8(234) break;
    case 235: LOOP_8(235) break;
    case 236: LOOP_8(236) break;
    case 237: LOOP_8(237) break;
    case 238: LOOP_8(238) break;
    case 239: LOOP_8(239) break;
    case 240: LOOP_8(240) break;
    case 241: LOOP_8(241) break;
    case 242: LOOP_8(242) break;
    case 243: LOOP_8(243) break;
    case 244: LOOP_8(244) break;
    case 245: LOOP_8(245) break;
    case 246: LOOP_8(246) break;
    case 247: LOOP_8(247) break;
    case 248: LOOP_8(248) break;
    case 249: LOOP_8(249) break;
    case 250: LOOP_8(250) break;
    case 251: LOOP_8(251) break;
    case 252: LOOP_8(252) break;
    case 253: LOOP_8(253) break;
    case 254: LOOP_8(254) break;
    case 255: LOOP_8(255) break;
   
      
  }  
  return acc;
}

static __attribute__((always_inline)) inline
bytes_t finalize(accumulator acc, bytes_t result0){
  /* How much to shift (right)?
    as much as acc.part index
    result0: 7
*/
  __m512i temp = _mm512_alignr_epi64(result0,result0,1);
  //__m512i temp2;
  result0 = _mm512_shrdi_epi64 (result0, temp, 7);

  temp = _mm512_alignr_epi64(acc.part1,acc.part1,1);
  result0 = _mm512_xor_epi64(result0,acc.part0);

  acc.part1 = _mm512_shrdi_epi64(acc.part1,temp,1);
  temp = _mm512_alignr_epi64(acc.part2,acc.part2,1);
  result0 = _mm512_xor_epi64(result0, acc.part1);
  
  acc.part2 = _mm512_shrdi_epi64(acc.part2,temp,2);
  temp = _mm512_alignr_epi64(acc.part3,acc.part3,1);
  result0 = _mm512_xor_epi64(result0, acc.part2);
   
  acc.part3 = _mm512_shrdi_epi64(acc.part3,temp,3);
  temp = _mm512_alignr_epi64(acc.part4,acc.part4,1);
  result0 = _mm512_xor_epi64(result0, acc.part3);

  acc.part4 = _mm512_shrdi_epi64(acc.part4,temp,4);
  temp = _mm512_alignr_epi64(acc.part5,acc.part5,1);
  result0 = _mm512_xor_epi64(result0, acc.part4);

  acc.part5 = _mm512_shrdi_epi64(acc.part5,temp,5);
  temp = _mm512_alignr_epi64(acc.part6,acc.part6,1);
  result0 = _mm512_xor_epi64(result0, acc.part5);

  acc.part6 = _mm512_shrdi_epi64(acc.part6,temp,6);
  temp = _mm512_alignr_epi64(acc.part7,acc.part7,1);
  result0 = _mm512_xor_epi64(result0, acc.part6);

  acc.part7 = _mm512_shrdi_epi64(acc.part7,temp,7);
  result0 = _mm512_xor_epi64(result0, acc.part7);

  return result0;
}


static void jump_formula_internal(size_t k,size_t ints_addr, ptrdiff_t bit_addr,char bit_addr_shift,bytes_t result0){
  ptrdiff_t i_base=0;
  accumulator accu = zero_acc();  
  //the same loop is executed p/8 + 1 times, however condition have memory access economies by getting them by int batchs, so we
  //exute the loop by batches of 7
  
  for (;i_base<=(ptrdiff_t)(p)-56;i_base+=56){
    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition

    for (int i=6;i>=0;i--){
      bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
      ints_addr++;
      accu = loop_once(accu, cond_bits>>(i*8), int_bits);  
    }
    bit_addr-=7;
  }

    uint64_t cond_bits = arr_geti(big_buffer,bit_addr-7)>>(bit_addr_shift); //get a pack of 56 condition
    cond_bits &= 0xFFFFFFFFFFFFFFFFUL<<(56-(p-i_base));
    
    for (int i=6;i>=0;i--){
      bytes_t int_bits=get_bytes(big_buffer,ints_addr);   //get corresponding bytes
      ints_addr++;
      accu = loop_once(accu, cond_bits>>(i*8), int_bits);  
    }

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
    mpz_t n2;
    mpz_init(n2);
    mpz_sub_ui(n2,n,1);
    n=n2;
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
  if (neg_n){
    mpz_clear(n);
  }
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
