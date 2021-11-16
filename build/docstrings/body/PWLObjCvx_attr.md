Indicates whether a variable has a convex piecewise-linear objective. Returns 0 if the piecewise-linear objective
function on the variable is non-convex. Returns 1 if the function is convex, or if the objective function on the
variable is linear.

This attribute is useful for isolating the particular variable that caused a continuous model with a piecewise-linear
objective function to become a MIP.