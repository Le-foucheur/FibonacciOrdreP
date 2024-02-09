#import "template.typ": *
#set page(numbering: "1/1", number-align: right)
#set heading(
  numbering: (..numbers) => {
    let n = numbers.pos().len();
    if n == 1 {numbering("1.", numbers.pos().at(0)) } 
    else if n == 2 { [Proposition ]; prop.step(); prop.display();":"}
    else if n == 3 { [*Preuve:*]} 
    else if n == 4 {[Conjecture: ]}
  },
)
#let local_link = (label, content) => link(label, {
  super[[#text(fill: blue)[#content]]]
})
#import "@preview/cetz:0.2.0" : *
#import "@preview/tablex:0.0.8" : *
#let pasc(n,k) = for x in range(0,n+1){
  let l = ()
  for y in range(k){
    if y <= x {
      l.push[#calc.binom(x,y)]
    } else { l.push[] }
  }
  l
}
#let lr(a,k,l, body) = {
  move(dy: -l*calc.sin(calc.atan(k))+a.last()+10pt, dx: l*calc.cos(calc.atan(k))+a.first())[#text(red)[#body]]
  line(stroke: red+.5pt, start: a, length: l, angle: calc.atan(-k))
}

#align(left)[Gaspar Daguet\ Julien Thillard\ Louwen Fricout]

#align(center, text(20pt)[TITRE])

#align(center,text[= Introduction])

La suite de Fibonacci a tout d'abord été étudiée en Inde via un problème de combinatoire dans des sortes de poèmes au V#super("e") siècle avant J.-C. par Pingala @Pingala notament. Puis, elle a été étudiée en Italie par le célèbre Léonard de Pise, plus connu sous le nom de Fibonacci, dans un problème sur la taille d'une population de lapins apparu dans son ouvrage #text(style: "italic")[Liber abbaci] @Liber en 1202.\
Cette suite aura toujours créé un certin engouement, et donc énormément de généralisation ont été créé comme les suites de Lucas @Lucas.\
Mais parmis toutes ces généralisations, beaucoup sont laissées de coté, et nous allons nous intéresser à l'une d'entre elles.

#align(center,text[= Définition])

Comme beaucoup le savent la suite de fibonacci est construite de manière récurrente en sommant les deux termes précédent et en prenant $F_0 = 1 et F_1 = 1$ (ou parfois $F_0 = 0 et F_1 = 1$), i.e. 
$ forall n in NN, F_n := cases(F_0 = F_1 = 1, F_(n+2) = F_(n+1) + F_n \, n>= 2) $
Pour généraliser cette suite nous n'allons pas sommer les deux termes précédents, mais le terme précédent et un terme se trouvant $p$ terme plus loin de ce premier terme et pour ce faire nous avons besoin que les $p$ premiers termes valent 1, i.e.
$ forall n, p in NN, F_n ^((p)) :=  cases(F_j^((p)) = 1\, "si " 0 <=j <= p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p) $
On nomme $p$ comme étant l'odre de la suite engendré et $(F^((p))_n)_(n in NN)$ la suite engendré pour un certain entier $p$


== Définition par récurrence équivalente #label("def2")
Nous pouvons considérer la définition suivante comme équivalente à la définition précédente : 
$ forall n, p in NN, F_(n-p)^((p)) =  cases(F_j^((p)) = 0\, "si" 0 <= j < p, F_n^((p)) = 1\, "si " n=p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p) $
Ce qui revient à décaler les termes de la suite de $p$ rangs.

=== \
Il est évident que les deux définitions sont équivalentes moyennant un décalage car les $p-1$ premier termes de la seconde définitions valent $0$ et le $p$-ième vaut $1$ #QED
#pagebreak()

#align(center, text[= Exemple de suite généré])

*Pour $p=0$:*\
Par la définition:
$ forall n in NN,  F_n^((0)) = cases(F_0^((0)) = 1, F_(n+1) = F_n +F_n = 2F_n) $
On retombe sur un suite géométrique de raison 2 et de premier terme 1, donc 
$ F_n^((0)) = 2^n $

*Pour $p=1$*\
On retombe par construction sur la suite de Fibonacci, donc 
$ forall n in NN,  F_n^((1)) = cases(F_0 = F_1 = 1, F_(n+2) = F_(n+1) + F_n) $
ou par la formule de binet $F_n^((1)) = 1/sqrt(5) (phi^(n+1) - phi'^(n+1))$ avec $phi = (1+sqrt(5))/2$ et $phi' = -1/phi$

*Pour $p=2$*\
Par la définition:
$ forall n in NN,  F_n^((2)) = cases(F_0 = F_1 = F_2 = 1, F_(n+3) = F_(n+2) + F_n) $
Ainsi on tombe sur la suite des vaches de Narayana @Narayana\
D'expression fonctionelle $F_n^((2)) = lambda^(n+2)/((lambda - nu)(lambda - mu)) + mu^(n+2)/((mu - nu)(mu - lambda)) + nu^(n+2)/((nu - lambda)(nu - mu))$ avec $lambda, mu$ et $nu$ les racines complexes du polynôme: $x^3-x^2-1$

// On verras bien
#align(center, text(size: 20pt)[A voir])
*Si $p --> +oo$*\
Par la définition, les $p$ premiers termes valent $1$, donc on pose 
$ forall n in NN,  F_n^((+oo)) = 1 $

#align(center, text[= Écriture fonctionelle des suites])

== Expression fonctionelle de $(Fnp)_(n in NN)$

Soit $R_1, R_2, . . . , R_(p+1)$ les racines complexes du polynômes $x^(p+1)-x^p-1$\
Alors $ Fnp = sum_(i=1)^(p+1) R^(n+p)/(display(product_(j =1\ j!=i)^(p+1)R_i - R_j)) $
#pagebreak()
=== \
Pour démontrer cette proposition nous utiliserons la seconde définition qui décale les termes de la suites avec $p$ zéros #local_link("def2", "def").\
Le théorème d'Alembert-Gauss nous assure que le polynôme caractéristique $x^(p+1) - x^p - 1$ possède $p+1$ racines complexes, notées: $R_1, R_2, ..., R_(p+1)$\
Ainsi $F_(n-p)^((p)) = display(sum_(i=1)^(p+1)) lambda_i R_i^n$ avec $lambda_i$ des constantes qu'il reste à déterminer.\
Pour cela, nous posons le système suivant grâce aux $p$ premiers termes qui sont définis :
$ cases(lambda_1 + lambda_2 + lambda_3 + ...  +lambda_(p+1) = F_0^((p)) = 0,
  lambda_1 R_1+ lambda_2 R_2+ lambda_3 R_3+ ...  +lambda_(p+1) R_(p+1)= F_1^((p)) = 0, 
  lambda_1 R_1^2+ lambda_2 R_2^2+ lambda_3 R_3^2+ ...  +lambda_(p+1) R_(p+1)^2= F_2^((p)) = 0,
  #h(1em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v,
  lambda_1 R_1^(p+1)+ lambda_2 R_2^(p+1)+ lambda_3 R_3^(p+1)+ ...  +lambda_(p+1) R_(p+1)^(p+1)= F_(p)^((p)) = 1) $
Ce qui est équivalent au système suivant :
$ mat(
1, 1, 1, ..., 1;
R_1, R_2, R_3, ..., R_(p+1);
R_1^2, R_2^2, R_3^2, ..., R_(p+1);
dots.v, dots.v, dots.v, dots.down, dots.v;
R_1^(p+1), R_2^(p+1), R_3^(p+1), ..., R_(p+1)^(p+1))mat(
lambda_1;
lambda_2;
lambda_3;
dots.v;
lambda_(p+1)) = mat(0;0;0;dots.v;1) $
On reconnaît la transposé d'une matrice de Vandermonde carré d'odre $p+1$ dont les coefficient sont deux à deux distincts. Cette matrice est donc inversible, notons $upright(A)$ cette matrice et $Lambda$ la matrice composée des coefficiens que l'on cherche. On a alors :
$ Lambda = upright(A)^(-1) mat(0;0;0;dots.v;1) $
Ainsi ce produit indique que l'on ne s'intéresse qu'à la dernière colonne de $upright(A)^(-1)$.\
De plus l'on sais que le $i$-ème coefficient de la dernière ligne de l'inverse d'une matrice de Vandermonde @InverVander (colonne ici, car on a la transposée) est égale à : $ 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $  \
#pagebreak()
Donc $ forall i in [|1;p+1|], lambda_i = 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $\
Ainsi en remplacent les $lambda_i$ dans $display(sum_(i=1)^(p+1)) lambda_i R_i^n$, on trouve bien:
$ F_(n-p)^((p)) = display(sum_(i=1)^(p+1))  R_i^n/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $
Ainsi en revenant à la définition de base :
$ Fnp = display(sum_(i=1)^(p+1))  R_i^(n+p)/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $ #QED

== Expression fonctionelle via le triangle de Pascale

$ forall n,p in NN, sum_(k=0)^(floor(n/(p+1))+1) binom(n-p k,k)  $

===
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
Par le principe de récurrence p-ième, $display(P(n): F^((p))_n = sum_(k=0)^(floor(n/(p+1))+1) binom(n - p k, k))$ #QED

*N.B:* pour $p=1$ et $p=0$, on retombe bien sur des résultats connues a savoir:
$ forall n in NN, F_n^((1)) = sum_(k=0)^(floor(n/2)+1) binom(n-k,k)  $
$ forall n in NN, sum_(k=0)^(n+1) binom(n,k) = 2^n = F_n^((0)) $

#pagebreak()

#grid(
  columns: (1fr,1fr),
  align(center)[
    *Pour $p = 2$*
    #tablex(
      columns: 10,
      auto-lines: false,
      ..pasc(9,10)
    )
  ],
  align(center)[
    *Pour $p=3$*
    #tablex(
      columns: 10,
      auto-lines: false,
      ..pasc(9,10)
    )
  ]
)
#line(start: (227pt,-13pt), end: (227pt,-22em))
#let k1 = 2.2
#lr((23pt,-205pt), k1, 10pt)[1]
#lr((23pt,-222pt), k1, 20pt)[1]
#lr((23pt,-238pt), k1, 35pt)[1]
#lr((23pt,-256pt), k1, 50pt)[2]
#lr((23pt,-270pt), k1, 60pt)[3]
#lr((23pt,-289pt), k1, 69pt)[4]
#lr((23pt,-304pt), k1, 85pt)[6]
#lr((23pt,-322pt), k1, 90pt)[9]
#lr((23pt,-339pt), k1, 123pt)[13]
#lr((23pt,-355pt), 2.1, 130pt)[19]

#v(-30.6em)
#let k2 = 3.2
#lr((250pt,-205pt), k2, 10pt)[1]
#lr((250pt,-222pt), k2, 25pt)[1]
#lr((250pt,-238pt), k2, 40pt)[1]
#lr((250pt,-256pt), k2, 53.5pt)[1]
#lr((250pt,-270pt), k2, 70pt)[2]
#lr((250pt,-289pt), k2, 90pt)[3]
#lr((250pt,-304pt), k2, 90pt)[4]
#lr((250pt,-322pt), k2, 113pt)[5]
#lr((250pt,-339pt), k2, 115pt)[7]
#lr((250pt,-355pt), k2, 130pt)[10]

#v(-32em)
On retrouve, comme pour Fibonacci, le faite que cela reviens à sommer les valeurs du triangle de Pascale avec une diagonale qui est de plus en plus penché en fonction de $p$, exemple ci-dessus

#align(center, text[= Sur les limites de quotients des $(Fnp)$])
Le ratio de deux termes successif de la suite de Fibonacci a toujours été porteur de mystère et d'isotérisme, néanmoins il en reste intéressant de s'y intéresser.\
C'est pourquoi nous allons voir les propriétés de deux généralisation de la limite de quotient.

*1#super("ère") généralisation:*\
Pour cette première généralisation, nous ne généraliserons par réelement le quotient, i.e. que nous allons nous intéréser à:
$ forall p in NN, lim_(n -> +oo) F_(n+1)^((p))/Fnp $

Regardons ce que cela donne pour certins $p$:

*Pour $p=0$*\
On sais que $forall n in NN, F_n^((0)) = 2^n$\
Ainsi 
$ F_(n+1)^((0))/F_(n)^((0)) = 2^(n+1)/2^n = 2 tend(n, +oo) 2 $

*Pour $p=1$*\
Il est connue que la limite du qotient la suite de Fibonacci tend vers $(1+sqrt(5))/2$
#pagebreak()

*Pour p >1*\
Au dela 1, il deviens difficile de calculer algébriquement le quotient, nous avons donc calculer informatiquement jusqu'à $p = 30$ en voici le tableau:
#grid(
  columns: (1fr,1fr),
  align(left)[
    Ainsi on peut traduire le tableau en un graphique
  #move(dx:-55pt)[
  #canvas( {
    plot.plot(
       axis-style: "left",
       size: (11,5.5),
       x-min: 0,
       x-max: 30,
       y-max: 2,
       y-min: .9,
       x-label:$p$,
       y-label:"",
       {
        plot.add(
          ((0,2),
            (1,1.618033989),
            (2,1.465571232),
            (3,1.380277569),
            (4,1.324717957),
            (5,1.285199033),
            (6,1.255422871),
            (7,1.232054631),
            (8,1.213149723),
            (9,1.197491434),
            (10,1.184276322),
            (11,1.172950750),
            (12,1.163119791),
            (13,1.154493551),
            (14,1.146854042),
            (15,1.140033937),
            (16,1.133902490),
            (17,1.128355940),
            (18,1.123310806),
            (19,1.118699108),
            (20,1.114464880),
            (21,1.110561598),
            (22,1.106950245),
            (23,1.103597835),
            (24,1.100476279),
            (25,1.097561494),
            (26,1.094832708),
            (27,1.092271899),
            (28,1.089863353),
            (29,1.087593296),
            (30,1.085449605)
            )
        )
        plot.add-hline(1)
       }
    )
  })
  Dont on voit clairement que le qotient tend vers 1\
  à partir de cette courbe on peut définir l'aproximation suivante:
  $ upright(A)_p = 1 + 1/((1+p)^(log_2(phi))) "avec" phi = (1+sqrt(5))/2 $
  Dont voici la courbe représentative : \
  #canvas({
    plot.plot(
      axis-style: "left",
       size: (11,5.5),
       x-min: 0,
       x-max: 30,
       y-max: 2,
       y-min: .9,
       x-label:$p$,
       y-label:"",
       legend: "legend.inner-north-east",
       legend-style: (stroke: 0pt, spacing: 1),
       {
        plot.add(domain: (0,30), x => 1+1/(calc.pow(x+1,calc.log(((1+calc.sqrt(5))/2), base:2))), label: $ upright(A)_p $)
        plot.add(
          ((0,2),
            (1,1.618033989),
            (2,1.465571232),
            (3,1.380277569),
            (4,1.324717957),
            (5,1.285199033),
            (6,1.255422871),
            (7,1.232054631),
            (8,1.213149723),
            (9,1.197491434),
            (10,1.184276322),
            (11,1.172950750),
            (12,1.163119791),
            (13,1.154493551),
            (14,1.146854042),
            (15,1.140033937),
            (16,1.133902490),
            (17,1.128355940),
            (18,1.123310806),
            (19,1.118699108),
            (20,1.114464880),
            (21,1.110561598),
            (22,1.106950245),
            (23,1.103597835),
            (24,1.100476279),
            (25,1.097561494),
            (26,1.094832708),
            (27,1.092271899),
            (28,1.089863353),
            (29,1.087593296),
            (30,1.085449605)
            ),
            label: "Quotient"
        )
        plot.add-hline(1)
       }
    )
  })
  #move(dx: 25pt, dy: -200pt)[$ R_p $]
  #move(dx: 25pt, dy: -526pt)[$ R_p $]
  ]],
  align(center)[
#table(
  columns: 2,
  inset: 4pt,
  align: (x, y) => (left, right).at(x),
  [$p$], [$ R_p $],
  [0],[2],
  [1],[1,618033989],
  [2],[1,465571232],
  [3],[1,380277569],
  [4],[1,324717957],
  [5],[1,285199033],
  [6],[1,255422871],
  [7],[1,232054631],
  [8],[1,213149723],
  [9],[1,197491434],
  [10],[1,184276322],
  [11],[1,172950750],
  [12],[1,163119791],
  [13],[1,154493551],
  [14],[1,146854042],
  [15],[1,140033937],
  [16],[1,133902490],
  [17],[1,128355940],
  [18],[1,123310806],
  [19],[1,118699108],
  [20],[1,114464880],
  [21],[1,110561598],
  [22],[1,106950245],
  [23],[1,103597835],
  [24],[1,100476279],
  [25],[1,097561494],
  [26],[1,094832708],
  [27],[1,092271899],
  [28],[1,089863353],
  [29],[1,087593296],
  [30],[1,085449605]
)])
#pagebreak()

==== \
le quotient noté $R_p$ peut s'écrire avec une sorte de fraction continue: 
$ R_p = 1 + 1/(1+ 1/(1+ 1/(dots))^p)^p $

*2#super("ième") généralisation*\
Pour mieux coller à la définition on peut au lieux de faire la limite du quotient entre deux termes successif, on peut faire la limite du quotient entre deux termes séparé par $p-1$ termes noté $Q_p$, i.e.:
$ forall p in NN, Q_p = lim_(n -> +oo) F_(n+p)^((p))/Fnp $

Regardons également ce que cela donne pour certaine valeur de $p$

*Pour $p = 0$*\
On a: $forall n in NN, F_n^((0)) = 2^n$\
Ainsi:
$ F_n^((0))/F_n^((0)) = 1 tend(n, +oo) 1 = Q_0 $

*Pour $p = 1$*\
Dans ce cas on retombe sur le même quotient étudier plus haut donc:
$ lim_(n -> +oo) F_(n+1)^((1))/F_(n)^((1)) = (1+sqrt(5))/2 = Q_1 $

*Pour p > 1*\
De même que pour la 1#super("er") généralisation, on a calculé le quotient jusqu'à $p=30$ compilé également en un tableau:
#grid(
  columns: (1fr, 1fr, 1fr),
  align(center)[
    #table(
      columns: 2,
      inset: 4pt,
      align: (x, y) => (left, right).at(x),
      [$p$], [quotient],
      [0], [1,0000],
      [1], [1,6180],
      [2], [2,1479],
      [3], [2,6297],
      [4],[3,0796],
      [5],[3,5063],
      [6],[3,9151],
      [7],[4,3093],
      [8],[4,6915],
      [9],[5,0635],
      [10],[5,4266],
    )
  ],
  align(center)[
    #table(
      columns: 2,
      inset: 4pt,
      align: (x, y) => (left, right).at(x),
      [11],[5,7820],
      [12],[6,1305],
      [13],[6,4728],
      [14],[6,8095],
      [15],[7,1411],
      [16],[7,4681],
      [17],[7,7908],
      [18],[8,1096],
      [19],[8,4247],
      [20],[8,7363],

    )
  ],
  align(center)[
    #table(
      columns: 2,
      inset: 4pt,
      align: (x, y) => (left, right).at(x),
      [21],[9,0447],
      [22],[9,3501],
      [23],[9,6527],
      [24],[9,9526],
      [25],[10,2499],
      [26],[10,5449],
      [27],[10,8375],
      [28],[11,1280],
      [29],[11,4164],
      [30],[11,7028],

    )
  ]
)
== \
Rappelle: on note $R_p$ le ratio de la première généralisation et $Q_p$ celle de la deuxième\
alors on a:
$ forall p in NN, (R_p)^p = Q_p $

=== \
Soit $p in NN$
$ F_(n+p)^((p)) / Fnp &= product_( k = n)^(n+p) F_(k+1)^((p)) / F_k^((p)) $
En passant à la limite dans l'égalité et comme le quotient de deux termes successif tend vers $R_p$, on obtient:
$ Q_p &= product_(k=n)^(n+p) R_p = product_(k=0)^(p) R_p = (R_p)^p  $
#QED


#align(center)[= Comportement de (#Fnp) sur $NN$]

== \
$ forall p in NN, forall n in [|0; p|], Fnp = 1 $

=== \
Ceci est immédiat via la définition

== \
$ forall p in NN, forall n in [|p+1; 2p+1|], Fnp = 1 + n - p $ \
i.e. que pour $n$ compris entre $p$ et $2p$, #Fnp se comporte comme une suite arithmétique de raison $1$ et de premier termes $1-p$

=== \
Soient $p in NN "et" n in [|p+1; 2p+1|]$\
Alors comme $n > p$ on peut aplliquer la formule de récurrence,\
Ainsi:
$ Fnp = F_(n-1)^((p)) + F_(n-p-1)^((p)) $
Or $p+1<= n  <= 2p+1$ donc $0 <= n - p -1 <= p$ donc $F_(n-p-1)^((p)) = 1$\
Donc:
$ Fnp = F_(n-1)^((p)) +1 $
Donc $(Fnp)_(p+1<=n<=2p+1)$ est suite arithmétique de raison $1$\ et de premier termes $F_(p+1)^((n)) = F_(p)^((p)) + F_(0)^((p)) = 2$\
Donc
$ forall p in NN, forall [|p+1; 2p+1|], F_(n)^((p)) = 1+n -p $
#QED

==== \
soit $k in NN$, les termes modulo 2 de $F_(k p+1)^((p))$ à $F_((k+1) p)^((p))$ forme un paterne\
Note: Ceci à déjà été démontrer dans les cas particuliers pour $k=0$ et $k=1$
#pagebreak()

#align(center)[= Dessin créé par $(Fnp)$ modulo $2$]
Si l'on prend sur une feuille à carreaux et que l'on mets dans la case d'indice $n,p$, le termes $F_n^((p))$ modulo 2, et que l'on colorise la dite case en noir ou en blanc si sa valeur est $1$ ou $0$, comme ci-dessous:

#figure(image("./fibo_suite.png"), caption: [dessin réalisé pour un nombre petit de cases])
On remarque en premier lieux que des motif apparaise entre les droites d'équations : $y = -x/n$ avec $n in NN^*$, ce qui reviens à la conjoecture précédente\
De plus si l'on prend des valeurs de $p$ et de $n$ bien plus grande on obtient:
#figure(image("./fibo_sequence.png"), caption: [dessins réalisé pour des valeurs bien plus grande])
On voit ici, un triangle de Sierpiński étiré de plus en plus vers le bas et arrondie vers des valeurs bien précises.\
On peut supposer que le triangle apparait dû au liens entre les suites de Fibonacci d'odre $p$ et le triangle de Pascale qui fait apparaitre le triangle par une contruction similaire.

#align(center)[= Propriétés divers des suites (#Fnp)]

== Formule du jump\
$ forall p, n, n' in NN, F_(n+n')^((p)) = F_n^((p)) F_(n')^((p)) + sum_(k=1)^p F_(n-k)^((p)) F_(n'+k-p-1)^((p)) $
(NB: on admet que, $forall p in NN, forall n in [|-p,-1|], F_n^((p)) = 0$,
ce qui est cohérent avec les généralisation au négatifs de chaque suite, et la formule de récurence.
On peut d'ailleur noter que cette formule (et sa preuve) restent valides dans cette généralisation aux n négatifs)
=== \
#let Fp(index) = $F_(index)^((p))$

Il est plus simple, pour l'objet de la preuve, de considerer la formule équivalente suivante:

$ forall p,i in NN, forall j in [|0,i|], Fp(i) = Fp(i-j) Fp(j) + sum_(k=1)^p Fp(i-j-k) Fp(j+k-p-1) $

(C'est la formule précédente en prenant $i=n+n'$ et $j=n'$)

Prouvons la proposition pour tout $p$ et $i$ par récurence sur $j$

Soit $p,i in NN$

Initialisation: $j=0$

$ Fp(i-0) Fp(0) + sum_(k=1)^p Fp(i-0-k) Fp(0+k-p-1) =
  Fp(i) times 1 + sum_(k=1)^p Fp(i-k) times 0 = Fp(i) $

Récurence: supposons que $exists j in NN, Fp(i) = Fp(i-j)  Fp(j) + sum_(k=1)^p Fp(i-j-k) Fp(j+k-p-1)$ et posons un tel $j$. On a alors:

$ &Fp(i) = Fp(i-j) Fp(j) + sum_(k=1)^p Fp(i-j-k) Fp(j+k-p-1) \
 &= (Fp(i-j-1) + Fp(i-j-p)) Fp(j) + sum_(k=0)^(p-1) Fp(i-j-k-1) Fp(j+k+1-p-1) \
 &= Fp(i-j-1) Fp(j) + underbrace(Fp(i-j-p) Fp(j+p+1-p-1),"peut rentrer comme terme p dans la somme")
   + sum_(k=1)^(p-1) Fp(i-j-k-1) Fp(j+k+1-p-1) + underbrace(Fp(i-j-1) Fp(j-p),"terme k=0 de la somme") \
  &= Fp(i-j-1) (Fp(j) + Fp(j-p)) + sum_(k=1)^p Fp(i-j-k-1) Fp(j+k+1-p-1) \
  &= Fp(i-(j+1)) Fp(j+1) + sum_(k=1)^p Fp(i-(j+1)-k) Fp((j+1)+k-p-1) $

On a alors prouvé que la formule est valable pour $j+1$, donc, par récurence sur $j$ (et comme cela est vrai pour tout $i$ et pour tout $p$):

  $ forall p,i in NN, forall j in [|0,i|], Fp(i) = Fp(i-j) Fp(j) + sum_(k=1)^p Fp(i-j-k) Fp(j+k-p-1) $  
#QED

* Application *

Cette formule, lorsque bien utilisée, permet de calculer en temps $O(p*log(n))$ le terme $n$ de la suite $F(p)$,
en ne manipulant que des entiers, et sans connaiscance préalable de la suite (par exemple les racines du polynôme caractéristique)
(voir algo_jump.c)








































#bibliography("Bibli.yml", style: "biomed-central", title: "References")