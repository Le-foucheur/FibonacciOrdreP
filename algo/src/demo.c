#include <stdio.h>
#include "external/gmp-6.3.0/gmp.h"
#include "lib_fibo_jump.h"
#include <stdlib.h>

int main(int argc, char* argv[]){
  if (argc==1) {
    printf("usage: %s p",argv[0]);
    exit(1);
  }
  ull p = 0;
  if(sscanf(argv[1],"%llu",&p)!=1){
    printf("number not formated as decimal");
    exit(1);
  }
  fibo_init_thread_pool(0);
  mpz_t n;
  mpz_init(n);
  mpz_array returned;
  
  while (mpz_inp_str(n,stdin,10)!=0) {
    returned=fibo(p, n);
    if (returned!=NULL) {
      for (ull i=0; i<p+1; i++) {
        putchar('\n');
        mpz_out_str(stdout,10,returned+i);
      }
    }
    putchar('\n');
    
  }
  exit(0);
}
