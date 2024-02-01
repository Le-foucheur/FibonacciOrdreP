#let prop = counter("Proposition")
#set heading(
  numbering: (..numbers) => {
    let n = numbers.pos().len();
    if n == 1 {numbering("1.", numbers.pos().at(0)) } 
    else if n == 2 { [Proposition ]; prop.step(); prop.display();":"}
    else if n == 3 { [*Preuve:*]} 
  },
)
#let et = "et" + h(5pt)
#let Fnp = $F_n^((p))$
#let defF = $forall n, p in NN, F_n ^((p)) :=  cases(F_j^((p)) = 1\, "si " 0 <=j <= p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p)$
#let tend(n, val) = $limits(-->)_(#n -> #val)$
#let QED = align(right, text[*Q.E.D.*])

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


== Définition par récurrence équivalente \
Nous pouvons considérer la définitions suivante comme équivalante à la définition de base:
$ forall n, p in NN, F_(n-p)^((p)) =  cases(F_j^((p)) = 0\, "si" 0 <= j < p, F_n^((p)) = 1\, "si " n=p, F_(n+p+1)^((p)) = F^((p))_(n+p) + F^((p))_n "si " n>p) $
Ce qui reviens à décaler les suites de $p$ termes 

=== \
Il est évident que les deux définitions sont équivalentes moyennant un décalage car les $p-1$ premier termes de la seconde définitions valent $0$ #QED
#pagebreak()

#align(center, text[= Exemple de suite généré])

*Pour $n=0$:*\
Par la définition:
$ F_n^((0)) = cases(F_0^((0)) = 1, F_(n+1) = F_n +F_n = 2F_n) $
On retombe sur un suite géométrique de raison 2 et de premier terme 1, donc 
$ F_n^((0)) = 2^n $

*Pour $n=1$*\
On retombe par construction sur la suite de Fibonacci, donc 
$ F_n^((1)) = cases(F_0 = F_1 = 1, F_(n+2) = F_(n+1) + F_n) $
ou par la formule de binet $F_n^(0) = 1/sqrt(5) (phi^(n+1) - phi'^(n+1))$ avec $phi = (1+sqrt(5))/2$ et $phi' = -1/phi$

*Pour $n=2$*\
Par la définition:
$ F_n^((2)) = cases(F_0 = F_1 = F_2 = 1, F_(n+3) = F_(n+2) + F_n) $
Ainsi on tombe sur la suite des vaches de Narayana @Narayana\
D'expretion fonctionelle $F_n^((2)) = lambda^(n+2)/((lambda - nu)(lambda - mu)) + mu^(n+2)/((mu - nu)(mu - lambda)) + nu^(n+2)/((nu - lambda)(nu - mu))$ avec $lambda, mu$ et $nu$ les racines complexes du polynômes: $x^3-x^2-1$

// On verras bien
#align(center, text(size: 20pt)[A voir])
*Si $p --> +oo$*\
Alors par la définition les $p$ premier termes valent $1$,donc on pose 
$ F_n^((+oo)) = 1 $

#align(center, text[= Écriture fonctionelle des suites])

== Expression fonctionelle de $(Fnp)_(n in NN)$

Soit $R_1, R_2, . . . , R_(p+1)$ les racines complexes du polynômes $x^(p+1)-x^p-1$\
Alors $ Fnp = sum_(i=1)^(p+1) R^(n+p)/(display(product_(j =1, j!=i)^(p+1)R_i - R_j)) $
#pagebreak()
=== \
Pour démontrer cette proposition nous utiliserons la seconde définitions qui décales les suites avec $p$ zéros.\
Le théorme d'Alembert-Gauss, nous assure que le polynômes caractéristique $x^(p+1) - x^p - 1$ possède $p+1$ racines complexes, notées: $R_1, R_2, ..., R_(p+1)$\
Ainsi $F_n^((p+n)) = display(sum_(i=1)^(p+1)) lambda_i R_i^n$ avec $lambda_i$ des constantes qu'ils restent à déterminer.\
Pour cela, nous posons le sitème suivant grâce aux $p$ premiers termes qui sont définis:
$ cases(lambda_1 + lambda_2 + lambda_3 + ...  +lambda_(p+1) = F_0^((p)) = 0,
  lambda_1 R_1+ lambda_2 R_2+ lambda_3 R_3+ ...  +lambda_(p+1) R_(p+1)= F_1^((p)) = 0, 
  lambda_1 R_1^2+ lambda_2 R_2^2+ lambda_3 R_3^2+ ...  +lambda_(p+1) R_(p+1)^2= F_2^((p)) = 0,
  #h(1em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v #h(3em) dots.v,
  lambda_1 R_1^(p+1)+ lambda_2 R_2^(p+1)+ lambda_3 R_3^(p+1)+ ...  +lambda_(p+1) R_(p+1)^(p+1)= F_(p)^((p)) = 1) $
Ce qui est équivalent au système suivant:
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
On reconnaît la transposé d'une matrice de Vandermonde carré d'odre $p+1$ dont les coefficient sont deux à deux disctins, donc cette matrice est inversible, notons $upright(A)$ cette matrice et $Lambda$ la matrices composé des coefficines que l'on cherches alors:
$ Lambda = upright(A)^(-1) mat(0;0;0;dots.v;1) $
Ainsi ce produit indique que l'on s'intéresse qu'à la dernière colonne de $upright(A)^(-1)$.\
De plus l'on sais que le $i$-èmes coefficient de la dernière ligne d'une matrice de Vandermonde @InverVander (colonne ici, car on a la transposé) est égale à: $ 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $  \
#pagebreak()
Donc $ forall i in [|1;p+1|], lambda_i = 1/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $\
Ainsi en remplacent les $lambda_i$ dans $display(sum_(i=1)^(p+1)) lambda_i R_i^n$, On trouve bien:
$ F_(n-p)^((p)) = display(sum_(i=1)^(p+1))  R_i^n/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $
Ainsi en revenant à la définition de base:
$ Fnp = display(sum_(i=1)^(p+1))  R_i^(n+p)/display(product_(j=1\ j!=i)^(p+1)R_i-R_j) $ #QED

= d
#bibliography("Bibli.yml", style: "biomed-central", title: "References")
