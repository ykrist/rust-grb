One unfortunate reality in MIP is that integer variables don't always take exact integral values. While this typically
doesn't create significant problems, in some situations the side-effects can be quite undesirable. The best-known
example is probably a trickle flow, where a continuous variable that is meant to be zero when an associated binary
variable is zero instead takes a non-trivial value. More precisely, given a constraint $y \leq M b$, where $y$ is a non-
negative continuous variable, $b$ is a binary variable, and $M$ is a constant that captures the largest possible value
of $y$, the constraint is intended to enforce the relationship that $y$ must be zero if $b$ is zero. With the default
integer feasibility tolerance, the binary variable is allowed to take a value as large as $1e-5$ while still being
considered as taking value zero. If the $M$ value is large, then the $M b$ upper bound on the $y$ variable can be
substantial.

Reducing the value of the `IntFeasTol` parameter can mitigate the effects of such trickle flows, but often at a
significant cost, and often with limited success. The `IntegralityFocus` parameter provides a better alternative.
Setting this parameter to 1 requests that the solver work harder to try to avoid solutions that exploit integrality
tolerances. More precisely, the solver tries to find solutions that are still (nearly) feasible if all integer variables
are rounded to exact integral values. We should say that the solver won't always succeed in finding such solutions, and
that this setting introduces a modest performance penalty, but the setting will significantly reduce the frequency and
magnitude of such violations.