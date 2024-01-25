*Preuve:*
Posons $display(P(n): F^((p))_n = sum_(k=0)^(ceil(n/p)) binom(n - p k, k))$

_Initialisation :_ Pour $n=0$, on a
$
sum_(k=0)^(ceil(n/p)) binom(n - p k, k)
= sum_(k=0)^0 binom(0 - p k, k)
= binom(0, 0)
= 1
$

_Hérédité :_ Soit $n in NN$ tel que $P(n)$ soit vraie.

$
F^((p))_(n+1)
&= F^((p))_(n-p) + F^((p))_(n) \
&= sum_(k=0)^(ceil((n-p)/p)) binom(n-p - p k, k) + sum_(k=0)^(ceil(n/p)) binom(n - p k, k) \
&= sum_(k=1)^(ceil((n-p)/p)+1) binom(n-p - p (k-1), k-1) + sum_(k=0)^(ceil(n/p)) binom(n - p k, k) \
&= sum_(k=0)^(ceil((n-p)/p)+1) binom(n - p k, k-1) + sum_(k=0)^(ceil(n/p)) binom(n - p k, k) \
&= sum_(k=0)^(ceil(n/p)) (binom(n - p k, k-1) + binom(n - p k, k)) \
&= sum_(k=0)^(ceil(n/p)) binom((n + 1) - p k, k) \
&= sum_(k=0)^(ceil((n+1)/p)) binom((n + 1) - p k, k) \
$

Donc $P(n+1)$ est vraie. #sym.square.filled
