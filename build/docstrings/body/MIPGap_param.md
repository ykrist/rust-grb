The MIP solver will terminate (with an optimal result) when the gap between the lower and upper objective bound is less
than `MIPGap` times the absolute value of the incumbent objective value. More precisely, if $z_P$ is the primal
objective bound (i.e., the incumbent objective value, which is the upper bound for minimization problems), and $z_D$ is
the dual objective bound (i.e., the lower bound for minimization problems), then the MIP gap is defined as

$gap = \vert z_P - z_D\vert / \vert z_P\vert$.

Note that if $z_P = z_D = 0$, then the gap is defined to be zero. If $z_P = 0$ and $z_D \neq 0$, the gap is defined to
be infinity.

For most models, $z_P$ and $z_D$ will have the same sign throughout the optimization process, and then the gap is
monotonically decreasing. But if $z_P$ and $z_D$ have opposite signs, the relative gap may increase after finding a new
incumbent solution, even though the absolute gap $\vert z_P - z_D\vert$ has decreased.

Note: Only affects mixed integer programming (MIP) models