Terminates as soon as the engine determines that the best bound on the objective value is at least as good as the
specified value. Optimization returns with an USER_OBJ_LIMIT status in this case.

Note that you should always include a small tolerance in this value. Without this, a bound that satisfies the intended
termination criterion may not actually lead to termination due to numerical round-off in the bound.

Note: Only affects mixed integer programming (MIP) models