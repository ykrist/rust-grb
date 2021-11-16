When solving a QCP model, the barrier solver terminates when the relative difference between the primal and dual
objective values is less than the specified tolerance (with a GRB_OPTIMAL status). Tightening this tolerance may lead to
a more accurate solution, but it may also lead to a failure to converge.

Note: Barrier only