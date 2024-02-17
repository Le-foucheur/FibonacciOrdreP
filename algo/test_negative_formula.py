#!/usr/bin/env python3

from math import comb, log

for p in range(1,100):
    n = 1000

    iteratif = [1] + [0] * p
    for i in range(n):
        iteratif.append(iteratif[i]-iteratif[i+1])

    formule = [1]

    def calcule(i):
        if i == 0:
            return 0
        sum = 0
        r = i % p
        q = i // p
        for k in range(q//p+1):
            if (q-1-k >= 0):
                sum += comb(q-1-k, r+p*k) * (-1)**(r+q+1 + (k if p%2==0  else 0))
        return sum

    for i in range(p+n):
        formule.append(calcule(i))

    less = [formule[i]-iteratif[i] for i in range(len(formule))]

    for i in range(n//p):
        print(less[p*i+1:p*i+p+1])


