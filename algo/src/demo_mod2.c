#include <stddef.h>
#include <stdio.h>
#include "external/gmp-6.3.0/gmp.h"
#include "lib_fibo_jump_mod2.h"
#include <stdlib.h>

extern unsigned char *fibo_output(size_t p, const char *n){
  mpz_t n_mpz;
  mpz_init(n_mpz);
  mpz_set_str(n_mpz, n, 10);
  unsigned char *output = fibo_mod2(p, n_mpz);
  mpz_clear(n_mpz);
  return output;
}

int main(int argc, char* argv[]){
  if (argc==1) {
    printf("usage: %s p",argv[0]);
    exit(1);
  }
  size_t p = 0;
  if(sscanf(argv[1],"%zu",&p)!=1){
    printf("number not formated as decimal");
    exit(1);
  }
  fibo2_init_thread_pool(0);
  mpz_t n;
  mpz_init(n);
  unsigned char* returned;
  
  while (mpz_inp_str(n,stdin,10)!=0) {
    returned=fibo_mod2(p, n);
    if (returned!=NULL) {
      for (size_t i=0; i<p+1; i++) {
        if (arr_getb(returned,p-i))
          putchar('#');
        else 
          putchar('.');
      }
    }
    putchar('\n');
  }
  exit(0);
}
