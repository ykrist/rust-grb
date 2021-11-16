Determines whether to use the homogeneous barrier algorithm. At the default setting (-1), it is only used when barrier
solves a node relaxation for a MIP model. Setting the parameter to 0 turns it off, and setting it to 1 forces it on. The
homogeneous algorithm is useful for recognizing infeasibility or unboundedness. It is a bit slower than the default
algorithm.

Note: Barrier only