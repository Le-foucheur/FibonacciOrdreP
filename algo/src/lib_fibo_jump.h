#ifndef LIB_FIBO_JUMP_H
#define LIB_FIBO_JUMP_H


#include <stdio.h>
#include "./external/gmp-6.3.0/gmp.h"

typedef __mpz_struct *mpz_array;
typedef unsigned long long ull ;


/* initialize the thread pool used to calulate fibonacci numbers
  MUST be called once at program start

  Return non-zero on error
*/
int fibo_init_thread_pool(size_t size);

/* Calcule le n-ième (et les p suivant) termes de la suite de fibonacci d'ordre p
*  Arguments: 
*    n: terme de la suite a calculer (pour des raisons d'optimisation, cette version ne supporte que les n>=0)
*    p: ordre de la suite de fibonacci (1 pour fibonaci standard) P doit etre au maximum ULL_MAX - 2. plus p est grand, plus les calculs consomeront de RAM
*    result: pointeur vers p+1 mpz_t valant Fp(n) a Fp(n+p)
*/
mpz_array fibo(ull p,mpz_t n);

#endif // LIB_FIBO_JUMP_H
