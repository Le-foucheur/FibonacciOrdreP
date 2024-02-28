#include "lib_fibo_jump.h"
#include "external/C-Thread-Pool/thpool.h"
#include "external/gmp-6.3.0/gmp.h"
#include <limits.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#elif MACOS
#include <sys/param.h>
#include <sys/sysctl.h>
#else
#include <unistd.h>
#endif

//Proto
int getNumCores(void);
void jump_formula(const mpz_array n_range,const mpz_array m_range,mpz_t result);
void threaded_jump_formulae(void* arg);
void shift_by(mpz_array src,ull dest_size,mpz_array dest,ull shift_by);
void ull2mpz(mpz_t z, ull value);
void threaded_jump_batch(long long int less_one);



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

#define LEFT_BIT (1L<<(sizeof(long long int)*8 -1))
#define RIGHT_BITS (ULLONG_MAX ^ (1L<<(sizeof(long long int)*8 -1)))


threadpool calcul_pool;
size_t p = 0;
mpz_array main_work_buffer = NULL;
ull main_size=0;
mpz_array work_buffer2 = NULL;


int fibo_init_thread_pool(size_t size){
  if (size==0) {
    size=getNumCores();
  }
  calcul_pool = thpool_init(size);
  if (calcul_pool==NULL) {
    return -1;
  }
  return 0;
}

/* Applique la formule du jump, et calcule Fp(n+m) etant donné dans n_range les Fp(n-p) a Fp(n) (inclusif, en ordre croissant), et de même dans m_range
*  result doit etre un entier initializé
*/
void jump_formula(const mpz_array n_range,const mpz_array m_range,mpz_t result){
  mpz_set_ui(result,0);
  mpz_addmul(result,n_range+p,m_range+p);
  for(ull i=0;i<p;i++){
    mpz_addmul(result,n_range+(p-i-1),m_range+i);
  }
}

void threaded_jump_formulae(void* arg){
    bool less_one = ((size_t)arg & LEFT_BIT)>>(sizeof(long long int)*8-1);
    ull i = (size_t)arg & RIGHT_BITS;
    jump_formula(main_work_buffer+main_size-p-1-((p-i+1)/2), main_work_buffer+main_size-p-1-less_one-((p-i)/2), work_buffer2+i);
}

/* Fonction séquentielle (donc non-multithredable), prenant en argument:
    -un buffer (src) de p+1 termes
    -un buffer de taille dest_size a remplir avec les termes de la suite + shift_by pour le dernier
    Le dernier terme sera donc celui de rang (dernier de src)+shift_by
*/
void shift_by(mpz_array src,ull dest_size,mpz_array dest,ull shift_by){
  if (shift_by<=dest_size-p-1){
    //mean there is a full copy of work_buffer_2, plus some terms to be calculated before and after
    for (ull i=0; i<=p; i++) {
      mpz_set(dest+dest_size-p-1-shift_by+i,src+i);  //copy
    }
    for (ull i=dest_size-shift_by; i<dest_size; i++) {
      mpz_add(dest+i,dest+i-1,dest+i-p-1);  //terms after
    }
    for (long long int i=dest_size-p-2-shift_by;i>=0;i--){
      mpz_sub(dest+i,dest+i+p+1,dest+i+p);  //term before
    }
  }
  else{
    if (shift_by>=dest_size) {
      //mean no terms are copied, and some are eventually to be calculated by cycling in work_buffer2
      ull i=p+1;
      while (shift_by>dest_size) {  //cycling
        if (i==p+1) {
          i=1;
          mpz_add(src,src,src+p);
          shift_by-=1;
        } else {
          mpz_add(src+i,src+i,src+i-1);
          i+=1;
          shift_by-=1;
        }
      }
      if (i==p+1) {  //first add with both operand in work_buffer2, result in main
            i=1;
            mpz_add(dest,src,src+p);
          } else {
            mpz_add(dest,src+i,src+i-1);
            i+=1;
          }
    
      for (ull j=1;j<p+1;j++){  //adds with one operand in main and one in buffer2
          if (i==p+1)
            i=0;
          mpz_add(dest+j,dest+j-1,src+i);
          i+=1;
      }
      for(i=p+1;i<dest_size;i++){  //adds with both operand in main
        mpz_add(dest+i,dest+i-1,dest+i-p-1);
      }
    } else {
      //mean there is a partial copy of work_buffer2, then some calculed terms
      ull to_copy = dest_size-shift_by;
      for (ull i=0;i<to_copy;i++){  //partial copy
        mpz_set(dest+i,src+p+1-to_copy+i);
      }
      for (ull i=0;i<p+1-to_copy;i++){  //calculated terms with one operand in each buffer
        mpz_add(dest+to_copy+i,dest+to_copy+i-1,src+i);
      }
      for(ull i=p+1;i<dest_size;i++){  //adds with both operand in main
        mpz_add(dest+i,dest+i-1,dest+i-p-1);
      }
      
    }
  }
}

void ull2mpz(mpz_t z, ull value)
{
    mpz_import(z, 1, -1, sizeof(ull), 0, 0, &value);
}

/*this function ASSUME dest is of size p+1, and source big enough to contain all terms
 nescessary to calculate the range twice further

  if less_one is set, will calculate as greatest term n*2-1 (with n the greater in source)
*/
void threaded_jump_batch(long long int less_one){
  less_one = less_one << (sizeof(long long int)*8-1);
  for (long long int i=p;i>=0;i--){

    thpool_add_work(calcul_pool, &threaded_jump_formulae, (void*)(i+less_one));
    //jump_formula(p, source+src_size-1-((p-i+1)/2), source+src_size-1-less_one-((p-i)/2), dest+i);
    
  }
  thpool_wait(calcul_pool);
}


mpz_array fibo(ull p_arg,mpz_t n){
  if (mpz_cmp_ui(n,0)<0){
    printf("negative integers not yet supported, abborting");
    return NULL;
  }
  if (p!=p_arg || main_work_buffer==NULL || work_buffer2==NULL){

    if (main_work_buffer!=NULL){
      for (ull i=0; i<main_size; i++) {
          mpz_clear(main_work_buffer+i);
        }
    }
    if (work_buffer2!=NULL) {
      for (ull i=0;i<p;i++){
        mpz_clear(work_buffer2+i);
      }
    }

    
    p = p_arg;
    main_size = p+1+(p+2)/2;
    if (main_size<p) {
      printf("OVERSIZED P: ABORTING");
      p=0;
      return NULL;
    }

    free(main_work_buffer);free(work_buffer2);
    main_work_buffer  = malloc(sizeof(__mpz_struct)*main_size);
    work_buffer2= malloc(sizeof(__mpz_struct)*(p+1));

    if (main_work_buffer==NULL||work_buffer2==NULL) {
      printf("NOT ENOUGH MEMORY: ABORTING");
      free(main_work_buffer);
      main_work_buffer=NULL;
      free(work_buffer2);
      work_buffer2=NULL;
      p=0;
      return NULL;
    }
  
  
    for (ull i=0; i<p; i++) {
      mpz_init_set_ui(work_buffer2+i,0);
    }
    mpz_init_set_ui(work_buffer2+p,1);
    for (ull i=0; i<main_size; i++){
      mpz_init(main_work_buffer+i);
    }
  } else {
    for (ull i=0; i<p; i++) {
      mpz_set_ui(work_buffer2+i,0);
    }
    mpz_set_ui(work_buffer2+p,1);
    
  }

  //to be optimized for low values ...
  //get biggest 2 pow in p
  unsigned int bits_p = 0;
  for (ull copy=p;copy!=0;copy >>= 1){
    bits_p++;
  }

  ull bits_n = mpz_sizeinbase(n,2);
  if (bits_n<=32 && bits_n<(ull)bits_p*2) {
    //we are just as fast by calculating them iteratively ...
    shift_by(work_buffer2, main_size, main_work_buffer,  mpz_get_ui(n));
    for (ull i=0; i<p+1; i++) {
      mpz_set(work_buffer2+i,main_work_buffer+main_size-p-1+i);
    }
   } else {
    //launch the big machine ...

    

    //mpz_t buffer2_val;
    //mpz_t main_buffer_val;
    //mpz_init_set_si(main_buffer_val,-1000);
    //mpz_init_set_ui(buffer2_val,0);

    //the point is to get to have work_buffer_2 filled up with value from n to n-p
    //to do that, we can: shift left (aka multiply by two) by using the jump formulae
    //add 1 (or a litle value) by using the regular shifting
    //initialize to a somewhat big value by initial shifting

    //How it work: we examine n as a bitfield from left to right (most to less significant bit)
    //We take  some first bit, and shift to that value, then, for each remaining bits, we:
    //shift left
    //add one if necessary (actually we do the two together)
    ull init=0;
    ull index=MIN(64,2*bits_p);
    index=MIN(index,bits_n);
    
    for (int i= index-1;i>=0;i--){
      init+= ((ull)(mpz_tstbit(n,bits_n-index+i)))<<i; 
    }
    index=bits_n-index-1;
    if (index==0){
      shift_by(work_buffer2 ,main_size , main_work_buffer, init+mpz_tstbit(n,0));
      //because the jump will do -1 if nedded instead of +1, we add 1 (*2 thx to jump) = +1
      threaded_jump_batch(mpz_tstbit(n,0));
    } else {

      shift_by(work_buffer2 ,main_size , main_work_buffer, init);
    
      if (index==ULLONG_MAX){
        for (ull i=0;i<p+1;i++){
          mpz_set(work_buffer2+i,main_work_buffer+i+main_size-p-1);
        }
      } else {
        while (index>1) {

          printf("jumps left to calculate: %llu\n",index);
          
          threaded_jump_batch(0);
          shift_by(work_buffer2, main_size, main_work_buffer, mpz_tstbit(n,index));
          index--;
        }
        threaded_jump_batch(0);
        shift_by(work_buffer2, main_size, main_work_buffer, mpz_tstbit(n,index)+mpz_tstbit(n,0));
        //because the jump will do -1 if nedded instead of +1, we add 1 (*2 thx to jump) = +1
        threaded_jump_batch(mpz_tstbit(n,0));
      }
  }
}

    printf("result calculated, printing ... \n");
    return work_buffer2;
}
