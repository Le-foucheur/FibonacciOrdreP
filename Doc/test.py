import numpy as np
import matplotlib.pyplot as plt

def fact(n):
    res = 1
    for i in range(1,n+1):
        res *= i
    return res

def H(x,k):
    res = 1
    for i in range(k):
        res =res *( x - i)
    return 1/fact(k) * res
    
def P(x,k,p):
    if k == 0: 
        return 1
    else:
        summ = 0
        for i in range (k):
            summ = summ + P((i+1)*p, i, p) * H(x - k * p, k - 1 - i)
        return H(x - k * p, k) + summ

def F(x,p):
    return P(x,int(np.floor(x/p)), p)

def R(x,d):
    return F(d+1, x)/F(d,x)

N = 5
a = 0.5
b = 30

x = [a + (b-a)/N * i for i in range(0, N+1)]
y = [R(x,100) for x in x]
ax = plt.gca()

ax.plot(x, y)
ax.grid(True)
ax.spines['left'].set_position('zero')
ax.spines['right'].set_color('none')
ax.spines['bottom'].set_position('zero')
ax.spines['top'].set_color('none')
plt.show()