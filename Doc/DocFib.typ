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

#align(center, text(20pt)[TITRE])

#align(center,text[= Introduction])

La suite de Fibonacci a tout d'abord été étudiée en Inde via un problème de combinatoire dans des sortes de poèmes au V#super("e") siècle avant J.-C. par Pingala @Pingala. Puis, elle a été étudiée en Italie par le célèbre Léonard de Pise, plus connu sous le nom de Fibonacci, dans un problème sur la taille d'une population de lapins apparu dans son ouvrage #text(style: "italic")[Liber abbaci] @Liber en 1202.\
Cette suite aura toujours créé un engouement, et donc énormément de généralisation ont été créé comme les suites de Lucas @Lucas.\
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
D'expression fonctionelle $F_n^((2)) = lambda^(n+2)/((lambda - nu)(lambda - mu)) + mu^(n+2)/((mu - nu)(mu - lambda)) + nu^(n+2)/((nu - lambda)(nu - mu))$ avec $lambda, mu$ et $nu$ les racines complexes du polynômes: $x^3-x^2-1$

// On verras bien
#align(center, text(size: 20pt)[A voir])
*Si $p --> +oo$*\
Par la définition, les $p$ premiers termes valent $1$, donc on pose 
$ forall n in NN,  F_n^((+oo)) = 1 $

#align(center, text[= Écriture fonctionelle des suites])

== Expression fonctionelle de $(Fnp)_(n in NN)$

Soit $R_1, R_2, . . . , R_(p+1)$ les racines complexes du polynômes $x^(p+1)-x^p-1$\
Alors $ Fnp = sum_(i=1)^(p+1) R^(n+p)/(display(product_(j =1, j!=i)^(p+1)R_i - R_j)) $
#pagebreak()
=== \
Pour démontrer cette proposition nous utiliserons la seconde définition qui décale les termes de la suites avec $p$ zéros #local_link("def2", "def").\
Le théorème d'Alembert-Gauss nous assure que le polynôme caractéristique $x^(p+1) - x^p - 1$ possède $p+1$ racines complexes, notées: $R_1, R_2, ..., R_(p+1)$\
Ainsi $F_n^((p+n)) = display(sum_(i=1)^(p+1)) lambda_i R_i^n$ avec $lambda_i$ des constantes qu'il reste à déterminer.\
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
De plus l'on sais que le $i$-ème coefficient de la dernière ligne d'une matrice de Vandermonde @InverVander (colonne ici, car on a la transposée) est égale à : $ 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $  \
#pagebreak()
Donc $ forall i in [|1;p+1|], lambda_i = 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $\
Ainsi en remplacent les $lambda_i$ dans $display(sum_(i=1)^(p+1)) lambda_i R_i^n$, on trouve bien:
$ F_(n-p)^((p)) = display(sum_(i=1)^(p+1))  R_i^n/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $
Ainsi en revenant à la définition de base :
$ Fnp = display(sum_(i=1)^(p+1))  R_i^(n+p)/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $ #QED

== Expression fonctionelle via le triangle de Pascale 
#pagebreak()

#align(center, text[= Sur les limites de quotients des $(Fnp)$])
Le ratio de deux termes successif de la suite de Fibonacci a toujours été porteur de mystère et d'isotérisme, néanmoins il en reste intéressant de s'y intéresser.\
C'est pourquoi nous allons voir les propriétés de deux généralisation de la limite de quotient.

*1#super("ère") généralisation:*\
Pour cette première généralisation, nous ne généraliserons par réelement le quotient, i.e. que nous allons nous intéréser à:
$ forall p in NN, lim_(n -> +oo) F_n^(p+1)/Fnp $

Regardons ce que cela donne pour certins $p$:

*Pour $p=0$*\
On sais que $forall n in NN, F_n^((0)) = 2^n$\
Ainsi 
$ F_(n+1)^((0))/F_(n)^((0)) = 2^(n+1)/2^n = 2 tend(n, +oo) 2 $

*Pour $p=1$*\
Il est connue que la limite du qotient la suite de Fibonacci tend vers $(1+sqrt(5))/2$

*Pour $p --> +oo$*\
On a définie pour $p --> +oo$, $ forall n in NN, F_n^((+oo)) =1 $
Donc 
$ F_(n+1)^((+oo))/F_n^((+oo)) = 1/1 = 1 tend(n,+oo) 1  $
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
  #move(dx: 25pt, dy: -200pt)[Quotient]
  #move(dx: 25pt, dy: -526pt)[Quotient]
  ]],
  align(center)[
#table(
  columns: 2,
  inset: 4pt,
  align: (x, y) => (left, right).at(x),
  [$p$], [quotient],
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

*Pour $p --> +oo$*\
On a: $forall n in NN, F_n^((+oo)) = 1$\
Ainsi:
$ F_n^((+oo))/F_n^((+oo)) = 1/1 = 1 tend(n, +oo) 1 = Q_(+oo) $

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
      [2], [2,6180],
      [3], [4,2361],
      [4], [6,8541],
      [5], [11,0902],
      [6], [17,9443],
      [7], [29,0344],
      [8], [46,9787],
      [9], [76,0132],
    )
  ],
  align(center)[
    #table(
      columns: 2,
      inset: 4pt,
      align: (x, y) => (left, right).at(x),
      [10], [122,9919],
      [11], [199,0050],
      [12], [321,9969],
      [13], [521,0019],
      [14], [842,9988],
      [15], [1 364,0007],
      [16], [2 206,9995],
      [17], [3 571,0003],
      [18], [5 777,9998],
      [19], [9 349,0001],
      [20], [15 126,9999],
    )
  ],
  align(center)[
    #table(
      columns: 2,
      inset: 4pt,
      align: (x, y) => (left, right).at(x),
      [21], [24 476,0000],
      [22], [39 603,0000],
      [23], [64 079,0000],
      [24], [103 682,0000],
      [25], [167 761,0000],
      [26], [271 443,0000],
      [27], [439 204,0000],
      [28], [710 647,0000],
      [29], [1 149 851,0000],
      [30], [1 860 498,0000],
    )
  ]
)
#pagebreak()
==== \
On remarque par le tableau ci-dessus que:
$ forall p in  NN, Q_p = phi^n "avec" phi = (1+sqrt(5))/2 $

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
On remarque en premier lieux que des motif apparaise entre les droites d'équations : $y = -x/n$ avec $n in NN^*$\
#pagebreak()
De plus si l'on prend des valeurs de $p$ et de $n$ bien plus grande on obtient:
#figure(image("./fibo_sequence.png"), caption: [dessins réalisé pour des valeurs bien plus grande])
On voit ici, un triangle de Sierpiński étiré de plus en plus vers le bas et arrondie vers des valeurs bien précises.\
On peut supposer que le triangle apparait dû au liens entre les suites de Fibonacci d'odre $p$ et le triangle de Pascale qui fait apparaitre le triangle par une contruction similaire.

#align(center)[= Propriétés divers des suites (Fnp)]

== Formule du jump\
$ forall p in NN n, n'>= p, F_(n+n')^((p)) = F_n^((p)) F_(n')^((p)) + sum_(k=1)^p F_(n-k)^((p)) F_(n'+k-p-1)^((p)) $

=== \













































#bibliography("Bibli.yml", style: "biomed-central", title: "References")
