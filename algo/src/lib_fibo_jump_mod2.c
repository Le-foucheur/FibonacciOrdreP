#include "lib_fibo_jump_mod2.h"
#include "external/C-Thread-Pool/thpool.h"
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
void jump_formula_internal(size_t k,size_t ints_addr, size_t bit_addr,uint64_t result);
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



unsigned char implem_array_getc(unsigned char* array,size_t index){  return array[-index];}
unsigned char implem_array_reverse_getc(unsigned char* array,size_t index){  return array[index];}

uint64_t implem_array_geti(unsigned char* array,size_t index){  return * ((uint64_t*)(array-index-7));}
uint64_t implem_array_reverse_geti(unsigned char* array,size_t index){  return * ((uint64_t*)(array+index));}

void implem_array_setc(unsigned char* array,size_t index,unsigned char set){  array[-index]=set;}
void implem_array_reverse_setc(unsigned char* array,size_t index,unsigned char set){  array[index]=set;}

void implem_array_seti(unsigned char* array,size_t index,uint64_t set){  *(uint64_t*)(array-index-7)=set;}
void implem_array_reverse_seti(unsigned char* array,size_t index,uint64_t set){  *(uint64_t*)(array+index)=set;}

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
/*uint64_t arr_get7c(unsigned char* array,size_t index){return arr_geti(array,index) & MASK_LOW;}

uint64_t arr_get_unaligned_7c(unsigned char* array,size_t index,unsigned char shift){
  uint64_t result=arr_geti(array,index);
  result>>=shift;
  result&= MASK_LOW;
  return result;
}*/

/* Applique la formule du jump, et calcule la range Fp(2n-8k **+2** ) à Fp(2n-8(k+7)+1 +2 ),placés dans little buffer, etant donné big_buffer rempli de suffiseament de Fp(n) et indices inférieurs
*
*/
void jump_formula_internal(size_t k,size_t ints_addr, size_t bit_addr,uint64_t result){
  for (size_t i=0;i<(p>>3);i++){
    uint64_t bits=arr_geti(big_buffer,ints_addr);
    for (unsigned char j=0; j<8; j++) {
      if (arr_getb(big_buffer,bit_addr-j)) {
        result^=bits>>j;
      }
    }
    bit_addr-=8;
    ints_addr++;
  }
  uint64_t bits=arr_geti(big_buffer,ints_addr);
    for (unsigned char j=0; j<(p&0b111); j++) {
      if (arr_getb(big_buffer,bit_addr-j)) {
        result^=bits>>j;
    }
  }
  arr_set7c(little_buffer, k, result);
}

void jump_formula_plus1(void* k_arg){
  size_t k=(size_t)k_arg;

  uint64_t result=0;
  size_t ints_addr;
  size_t bit_addr;
  
  if (k==0){
    //n=-1, n_p = 0

    if (arr_getb(big_buffer, 0)) { //0 bc that is the value of n_p
        result = arr_geti(big_buffer, 0);
        result =  result<<1 | ((result&1) ^ arr_getb(big_buffer, p) );
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p; //p+n_p
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1...
    size_t n_p = ((k+1)/2)*8;

    if (arr_getb(big_buffer, n_p))
      result = arr_geti(big_buffer,n/8)>>(n%8);
    
    ints_addr=(n+1)/8;
    bit_addr=n_p+p;
  }
  jump_formula_internal(k,ints_addr , bit_addr,result);
}

/* Applique la formule du jump, et calcule la range Fp(2n-8k **+0** ) à Fp(2n-8(k+7)+1 +0 ),placés dans little buffer, etant donné big_buffer rempli de suffiseament de Fp(n) et indices inférieurs
*  Valid if and only if p>=7
*/


void jump_formula(void* k_arg){
  size_t k=(size_t)k_arg;

  uint64_t result=0;
  size_t ints_addr;
  size_t bit_addr;
  
  if (k==0){
    //to be easier in int reading,n must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    //n=-1, n_p = 1 
    if (arr_getb(big_buffer, 1)) { //1 bc that is the value of n_p
        result = arr_geti(big_buffer, 0);
        result =  result<<1 | ((result&1) ^ arr_getb(big_buffer, p) );
    }//           for n = -1, left shift, and we calculate the rightmost
    ints_addr=0; //(n+1)/8
    bit_addr=p+1; //p+n_p
    
  } else {
    size_t n=(k/2)*8 - 1; //to be easier in int reading, must be a multiple of 8 less 1... so we trick and add one to n_p in exchange
    size_t n_p = ((k+1)/2)*8 +1 ;
  
    if (arr_getb(big_buffer, n_p))
      result = arr_geti(big_buffer,n/8)>>(n%8);
    
    ints_addr=(n+1)/8;
    bit_addr=n_p+p;
  }
  jump_formula_internal(k,ints_addr , bit_addr,result);
  
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


unsigned char* fibo_mod2(size_t p_arg,mpz_t n){
  if (mpz_cmp_ui(n,0)<0){
    printf("negative integers not yet supported, abborting");
    return NULL;
  }
  size_t min_valid_size = MIN(2*p_arg+3,p_arg+36+p_arg/2) ;
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
