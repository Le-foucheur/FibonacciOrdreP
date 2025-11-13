#let prop = counter("Proposition")
#let et = "et" + h(5pt)
#let Fnp = $F_n^((p))$
#let defF = $forall n, p in NN, F_n^((p)) := cases(F_j^((p)) = 1\, "si " 0 <=j <= p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p)$
#let tend(n, val) = $limits(-->)_(#n -> #val)$
#let QED = align(right, text[*Q.E.D.*])
#let pasc(n, k) = for x in range(0, n + 1) {
  let l = ()
  for y in range(k) {
    if y <= x {
      l.push[#calc.binom(x, y)]
    } else { l.push[] }
  }
  l
}
#let lr(a, k, l, body) = {
  move(dy: -l * calc.sin(calc.atan(k)) + a.last() + 10pt, dx: l * calc.cos(calc.atan(k)) + a.first())[#text(red)[#body]]
  line(stroke: red + .5pt, start: a, length: l, angle: calc.atan(-k))
}
#let Rf = text(font: "FreeMono")[#str.from-unicode(0x16A0)] 