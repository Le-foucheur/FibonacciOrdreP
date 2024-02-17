#include "lib_fibo_jump.h"
#include "external/c_threads_lib/thread_pool/thread_pool.h"
#include "external/gmp-6.3.0/gmp.h"
#include <gmp.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>


/* Applique la formule du jump, et calcule Fp(n+m) etant donné dans n_range les Fp(n-p) a Fp(n) (inclusif, en ordre croissant), et de même dans m_range
*  result doit etre un entier initializé
*/
void jump_formula(ull p,const mpz_array n_range,const mpz_array m_range,mpz_t result){
  mpz_set_ui(result,0);
  mpz_addmul(result,n_range+p,m_range+p);
  for(ull i=0;i<p;i++){
    mpz_addmul(result,n_range+(p-i-1),m_range+i);
  }
}

/* Fonction séquentielle (donc non-multithredable), prenant en argument:
    -un buffer (src) de p+1 termes
    -un buffer de taille dest_size a remplir avec les termes de la suite + shift_by pour le dernier
    Le dernier terme sera donc celui de rang (dernier de src)+shift_by
*/
void shift_by(ull p,mpz_array src,ull dest_size,mpz_array dest,ull shift_by){
  if (shift_by<=dest_size-p-1){
    for (ull i=0; i<=p; i++) {
      mpz_set(dest+dest_size-p-1-shift_by+i,src+i);
    }
    for (ull i=dest_size-shift_by; i<dest_size; i++) {
      mpz_add(dest+i,dest+i-1,dest+i-p);
    }
    for (ull i=dest_size-p-2-shift_by;i>=0;i++){
      mpz_sub(dest+i,dest+i+p+1,dest+i+p);
    }
  }
  else{
    ull i=p+2;
    while (shift_by>dest_size-p-1) {
      if (i==p+2) {
      
      }
    } 
  }
}

void ull2mpz(mpz_t z, ull value)
{
    mpz_import(z, 1, -1, sizeof(ull), 0, 0, &value);
}

mpz_array fibo(ull p,mpz_t n){
  if (mpz_cmp_ui(n,0)<0){
    printf("negative integers not yet supported, abborting");
    return NULL;
  }
  ull main_buffer_size = p+1+(p+2)/2;
  if (main_buffer_size<p) {
      printf("OVERSIZED P: ABORTING");
      return NULL;
  }
  mpz_array main_work_buffer  = malloc(sizeof(__mpz_struct)*main_buffer_size);
  mpz_array work_buffer_2= malloc(sizeof(__mpz_struct)*(p+1));

  if (main_work_buffer==NULL||work_buffer_2==NULL) {
    printf("NOT ENOUGH MEMORY: ABORTING");
    free(main_work_buffer);
    free(work_buffer_2);
    return NULL;
  }
  
  mpz_t buffer2_val;
  mpz_t main_buffer_val;
  mpz_init_set_si(main_buffer_val,-1000);
  mpz_init(buffer2_val);
  ull2mpz(buffer2_val, p);
  

  
  
  for (ull i=0; i<p+1; i++) {
    mpz_init_set_ui(work_buffer_2+i,1);
  }
  
  
}
