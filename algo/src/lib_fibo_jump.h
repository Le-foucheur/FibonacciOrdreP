#ifndef LIB_FIBO_JUMP_H
#define LIB_FIBO_JUMP_H


#include <gmp.h>

typedef __mpz_struct *mpz_array;
typedef unsigned long long ull ;

/* Calcule le n-ième terme de la suite de fibonacci d'ordre p, en stockant les résultats pour accélérer des éventuels calculs ultérieur 
*  Arguments: 
*    n: terme de la suite a calculer (pour des raisons d'optimisation, cette version ne supporte que les n>=0)
*    p: ordre de la suite de fibonacci (1 pour fibonaci standard)
*    result: pointeur vers p+1 mpz_t valant Fp(n) a Fp(n+p)
*/
mpz_array fibo(ull p,mpz_t n);

#endif // LIB_FIBO_JUMP_H
