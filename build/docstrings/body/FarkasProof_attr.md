Together, attributes `FarkasDual` and `FarkasProof` provide a certificate of infeasibility for the given infeasible
problem. Specifically, `FarkasDual` can be used to form the following inequality from the original constraints that is
infeasible within the bounds of the variables:

$\lambda^tAx \leq \lambda^tb.$

This Farkas constraint is valid, because $\lambda_i \geq 0$ if the $i$-th constraint has a $\leq$ sense and $\lambda_i
\leq 0$ if the $i$-th constraint has a $\geq$ sense.

Let

$\bar{a} := \lambda^tA$

be the coefficients of this inequality and

$\bar{b} := \lambda^tb$

be its right hand side. With $L_j$ and $U_j$ being the lower and upper bounds of the variables $x_j$ we have $\bar{a}_j
\geq 0$ if $U_j = \infty$, and $\bar{a}_j \leq 0$ if $L_j = -\infty$.

The minimum violation of the Farkas constraint is achieved by setting $x^*_j := L_j$ for $\bar{a}_j > 0$ and $x^*_j :=
U_j$ for $\bar{a}_j < 0$. Then, we can calculate the minimum violation as

$\beta := \bar{a}^tx^* - \bar{b} = \sum\limits_{j:\bar{a}_j>0}\bar{a}_jL_j + \sum\limits_{j:\bar{a}_j<0}\bar{a}_jU_j -
\bar{b}$

where $\beta>0$.

The `FarkasProof` attribute provides $\beta$, and the `FarkasDual` attributes provide the $\lambda$ multipliers for the
original constraints.

These attributes are only available when parameter InfUnbdInfo is set to 1.