# Suite de Fibonacci généralisée: études et programmes
Auteurs: Gaspar Daguet, Julien Thillard, Louwen Fricout, Albin Chaboissier

Ce repo github contient des résultats de recherche sur une généralisation de la suite de Fibonacci,
ainsi que des programmes pour calculer et prévisualiser leur termes

Nous tenons a remercier le projet Grid5000, qui nous a permis d'utiliser leur supercalculateurs afin de lancer les programmes de calcul pour de très grand termes de ces suites

La suite de suite est définie telle que F^p_n vaut 1 pour les p+1 premiers éléments, puis F^p_(n+1) = F^p_n + F^p__(n-p)

Plus de détails sur la suite, et les résultats de recherches sont disponibles dans
[Doc/DocFib.pdf](https://github.com/Le-foucheur/FibonacciOrdreP/blob/main/Doc/DocFib.pdf)
(Auteur principal: Gaspar Daguet)

Dans le dossier [algo](https://github.com/Le-foucheur/FibonacciOrdreP/tree/main/algo)
se trouvent 2 programmes permettant de calculer les termes des suite et ces termes modulo 2, sous formes de librairies C, et de petit programmes TUI leur faisant appel
(Auteur principal: Louwen Fricout)

Dans le dossier [fibo_render](https://github.com/Le-foucheur/FibonacciOrdreP/tree/main/fibo_render)
se trouvent un programme de visualisation (et génération d'image) des termes modulo 2, avec augmentation des p sur l'axe vertical, et n sur l'axe horizontal
(Auteur principal: Julien Thillard)

Les détails de compilation et d'utilisations de ces programmes se trouvent dans leur README respéctifs
