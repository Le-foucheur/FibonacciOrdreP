*Preuve:*
Posons $display(P(n): F^((p))_n = sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k))$

_Initialisation :_ Pour $n<=p$, on a
$
  sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k)
  = sum_(k=0)^1 binom(n - p k, k)
  = underbrace(binom(n, 0), =1) + underbrace(binom(n-p, 1), n-p<=0 "donc" 0)
  = 1
$

_Hérédité :_ Soit $n in NN$ tel que $forall k in [|0, n|], P(k)$ soit vraie.

$
  F^((p))_(n+1)
    &= F^((p))_(n-p) + F^((p))_(n) \
    &= sum_(k=0)^(floor((n-p)/(p+1))+1) binom(n-p - p k, k) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
    &= sum_(k=1)^(floor((n-p)/(p+1))+2) binom(n-p - p (k-1), k-1) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
$
Or $display(binom(n, -1) = 0)$ donc on peut décaler l'indice de la première
somme à $k=0$ :
$
    F^((p))_(n+1)&= sum_(k=0)^(floor((n-p)/(p+1))+2) binom(n - p k, k-1) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
$
On peut alors essayer de regrouper les deux sommes :

$floor((n-p)/(p+1))+2 = floor((n+p+2)/(p+1))
"et"
floor((n)/(p+1))+1 = floor((n+p+1)/(p+1))
"donc" floor((n-p)/(p+1))+2 >= floor((n)/(p+1))+1 \ $

On souhaite donc montrer que $floor((n-p)/(p+1))+2 > n-p(floor((n-p)/(p+1))+2)$ :
on a
$
  (n-p)/(p+1) - 1 < floor((n-p)/(p+1))
  &<=> (p+1)(floor((n-p)/(p+1))+2) > n-p + (p+1) \
  &<=> -(p+1)(floor((n-p)/(p+1))+2) < -n-1 \
  &<=> n-(p+1)(floor((n-p)/(p+1))+2) < -1 \
  &<=> n-p(floor((n-p)/(p+1))+2) < -1 +floor((n-p)/(p+1))+2 \
  &<=> n-p(floor((n-p)/(p+1))+2) < floor((n-p)/(p+1)) + 2 \
$
Donc $display(binom(n-floor((n-p)/(p+1))+2, floor((n-p)/(p+1))+2)) = 0$, ce qui permet d'utiliser $floor((n-p)/(p+1))+2$ comme indice commun au deux sommes, qu'on peut donc regrouper :

$
    F^((p))_(n+1)&= sum_(k=0)^(floor((n-p)/(p+1))+2) (binom(n - p k, k-1) + binom(n - p k, k)) \
    &= sum_(k=0)^(floor((n-p)/(p+1)+1)+1) binom((n + 1) - p k, k) \
    &= sum_(k=0)^(floor((n+1)/(p+1))+1) binom((n + 1) - p k, k) \
$

Donc $P(n+1)$ est vraie.\
Par le principe de récurrence p-ième, $display(P(n): F^((p))_n = sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k))$ #sym.square.filled
