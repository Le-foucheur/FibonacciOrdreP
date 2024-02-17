#include "lib_fibo_jump.h"
#include "external/c_threads_lib/thread_pool/thread_pool.h"
#include <gmp.h>
#include <stdatomic.h>
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

mpz_array fibo(ull p,mpz_t n){
  mpz_array work_buffer = malloc(sizeof(__mpz_struct)*(p+1))
}
