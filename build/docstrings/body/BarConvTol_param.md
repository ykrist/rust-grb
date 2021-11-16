The barrier solver terminates when the relative difference between the primal and dual objective values is less than the
specified tolerance (with a GRB_OPTIMAL status). Tightening this tolerance often produces a more accurate solution,
which can sometimes reduce the time spent in crossover. Loosening it causes the barrier algorithm to terminate with a
less accurate solution, which can be useful when barrier is making very slow progress in later iterations.

Note: Barrier only