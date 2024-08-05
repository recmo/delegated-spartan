#!/usr/bin/env sage
import itertools

# Bn254 scalar field
p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
F = GF(p)
N = 16

def check_matrix(m):
    if m.density() < 1 or not m.is_invertible():
        return False
    for i in range(1, 2 * m.nrows() + 1):
        if not (m^i).characteristic_polynomial().is_irreducible():
            return False
    return True

solutions = []
for diag in itertools.combinations(range(0, N+2), N):
    m = ones_matrix(F, N) + diagonal_matrix(diag)
    if check_matrix(m):
        print(diag)
        solutions.append(diag)

if not solutions:
    print("No solutions found")
    exit()

solutions = sorted(solutions, key=lambda x: max(x))
diag = solutions[0]
m = ones_matrix(F, N) + diagonal_matrix(diag)

print(solutions[0])
print(m)
assert check_matrix(m)
