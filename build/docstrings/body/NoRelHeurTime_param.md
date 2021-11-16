Limits the amount of time (in seconds) spent in the NoRel heuristic. This heuristic searches for high-quality feasible
solutions before solving the root relaxation. It can be quite useful on models where the root relaxation is particularly
expensive.

Note that this parameter will introduce non-determinism - different runs may take different paths. Use the
`NoRelHeurWork` parameter for deterministic results.

Note: Only affects MIP models