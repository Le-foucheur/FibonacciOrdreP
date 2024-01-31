*Preuve:*
Posons $display(P(n): F^((p))_n = sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k))$

_Initialisation :_ Pour $n=0$, on a
$
sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k)
= sum_(k=0)^1 binom(0 - p k, k)
= underbrace(binom(0, 0), =1) + underbrace(binom(-p, 1), =0)
= 1
$

_Hérédité :_ Soit $n in NN$ tel que $forall k in [|0, n|], P(k)$ soit vraie.

$
F^((p))_(n+1)
&= F^((p))_(n-p) + F^((p))_(n) \
&= sum_(k=0)^(floor((n-p)/(p+1))+1) binom(n-p - p k, k) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
&= sum_(k=1)^(floor((n-p)/(p+1))+2) binom(n-p - p (k-1), k-1) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
$
Or $display(binom(n, -1) = 0)$ donc on peut décaler l'indice de la première somme à $k=0$ :
$
&= sum_(k=0)^(floor((n-p)/(p+1))+2) binom(n - p k, k-1) + sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k) \
$
On peut alors essayer de regrouper les deux sommes :
$
&= sum_(k=0)^(floor((n-p)/(p+1))+2) (binom(n - p k, k-1) + binom(n - p k, k)) \
&= sum_(k=0)^(floor((n-p)/(p+1)+1)+1) binom((n + 1) - p k, k) \
&= sum_(k=0)^(floor((n+1)/(p+1))+1) binom((n + 1) - p k, k) \
$

Donc $P(n+1)$ est vraie. \
Par le principe de récurrence forte, $display(P(n): F^((p))_n = sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k))$ #sym.square.filled
