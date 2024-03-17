#include <stddef.h>
#include <stdio.h>
#include "external/gmp-6.3.0/gmp.h"
#include "lib_fibo_jump_mod2.h"
#include <stdlib.h>

int main(int argc, char* argv[]){
  if (argc==1) {
    printf("usage: %s p [n] [cores]",argv[0]);
    exit(1);
  }
  size_t p = 0;
  if(sscanf(argv[1],"%zu",&p)!=1){
    printf("number not formated as decimal");
    exit(1);
  }
  if(fibo_mod2_initialization(p)) exit(1);
  if (argc>3){
    int temp;
    if(sscanf(argv[3],"%u",&temp)!=1){
      printf("number not formated as decimal");
      exit(1);
    }
    
    fibo2_init_thread_pool(temp);
  } else {
    fibo2_init_thread_pool(0);
  }
  mpz_t n;
  mpz_init(n);
  unsigned char* returned;
  if (argc>2){
    //performance testing mode
    mpz_set_str(n,argv[1],10);
    returned=fibo_mod2(p, n);
    if (returned!=NULL) {
      for (size_t i=0; i<5; i++) {
        if (arr_getb(returned,p-i))
          putchar('#');
        else 
          putchar('.');
      }
    }
    
  } else {
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
  }}
  exit(0);
}
