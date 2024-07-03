# Delegated Spartan

$$
\gdef\mat#1{\mathrm{#1}}
\gdef\p#1{\left({#1}\right)}
\gdef\ceil#1{\left\lceil{#1}\right\rceil}
\gdef\forall{\mathop{\huge ∀}\limits}
$$

Experiment to get R1CS (and in particular Circom circuits) efficiently verified on low-power mobile devices. The main plan is to use SpartanNIZK over the BN254 scalar field and then send the $<100\,\mathrm{kB}$ proof to a server for wrapping in a succinct system.

## SpartanNIZK

Spartan ([S19]) is a transparant zkSNARK for R1CS. Recal an R1CS instance over a field $𝔽$ with $n$-sparse $m×m$ matrices $\mat A, \mat B, \mat C$ such that a $z=(1,\mathsf{pub},\mathsf{priv})$ satisfies iff
$(\mat A⋅ z) ∘(\mat B ⋅ z) = \mat C ⋅ z$.
We convert this to a [sumcheck zero testing] statement
$$
\forall_{x∈\{0,1\}^s}0=
\p{\sum_{y∈\{0,1\}^s}\widetilde A(x,y)⋅\widetilde z(y)}⋅
\p{\sum_{y∈\{0,1\}^s}\widetilde B(x,y)⋅\widetilde z(y)}\\[.5em]-
\sum_{y∈\{0,1\}^s}\widetilde C(x,y)⋅\widetilde z(y)
$$
where $\widetilde\square$ denotes a multilinear extension and $s=\ceil{\log_2 m}$. Batching the inner sumchecks, it takes two sumchecks to reduce this to
$$
(r_A⋅\widetilde A(r_x, r_y) +
r_B⋅\widetilde B(r_x, r_y) +
r_C⋅\widetilde C(r_x, r_y)) ⋅
\widetilde z(r_y)
$$
For $\widetilde z$ the prover provides a hiding polynomial commitment to $\mathsf{priv}$ up front and reveals it at $r_y$ so that the verifier can compute $\widetilde z(r_y)$. The verifier knows $\widetilde A, \widetilde B, \widetilde C$ and can evaluate it directly.

[sumcheck zero testing]: /24/sumcheck-gkr#zero-testing

## References

* [S19] Srinath Setty (2019). Spartan: Efficient and general-purpose zkSNARKs without trusted setup.

[S19]: https://eprint.iacr.org/2019/550
