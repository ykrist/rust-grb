Limits the amount of work spent in the NoRel heuristic. This heuristic searches for high-quality feasible solutions
before solving the root relaxation. It can be quite useful on models where the root relaxation is particularly
expensive.

The work metric used in this parameter is tough to define precisely. A single unit corresponds to roughly a second, but
this will depend on the machine, the core count, and in some cases the model. You may need to experiment to find a good
setting for your model.

Note: Only affects MIP models