import math
import time

start = time.perf_counter()
n = 1_000_000
prime = [True] * (n + 1)
prime[0] = False
prime[1] = False

for p in range(2, int(math.sqrt(n)) + 1):
    if prime[p]:
        for i in range(p*p, n + 1, p):
          prime[i] = False

        # this is WAY faster, but Jed doesn't have a bytecode like it so...
        # prime[p*p::p] = [False] * len(prime[p*p::p])


for i in range(2, n+1):
    if prime[i]:
        print(i)
print((time.perf_counter() - start) * 1000)
