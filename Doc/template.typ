#let prop = counter("Proposition")
#let et = "et" + h(5pt)
#let Fnp = $F_n^((p))$
#let defF = $forall n, p in NN, F_n ^((p)) :=  cases(F_j^((p)) = 1\, "si " 0 <=j <= p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p)$
#let tend(n, val) = $limits(-->)_(#n -> #val)$
#let QED = align(right, text[*Q.E.D.*])
