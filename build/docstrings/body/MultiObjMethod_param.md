When solving a continuous multi-objective model using a hierarchical approach, the model is solved once for each
objective. The algorithm used to solve for the highest priority objective is controlled by the `Method` parameter. This
parameter determines the algorithm used to solve for subsequent objectives. As with the `Method` parameters, values of 0
and 1 use primal and dual simplex, respectively. A value of 2 indicates that warm-start information from previous solves
should be discarded, and the model should be solved from scratch (using the algorithm indicated by the `Method`
parameter). The default setting of -1 usually chooses primal simplex.

Note: Only affects continuous multi-objective models