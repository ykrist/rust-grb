Limits degenerate simplex moves. These moves are performed to improve the integrality of the current relaxation
solution. By default, the algorithm chooses the number of degenerate move passes to perform automatically.

The default setting generally works well, but there can be cases where an excessive amount of time is spent after the
initial root relaxation has been solved but before the cut generation process or the root heuristics have started. If
you see multiple 'Total elapsed time' messages in the log immediately after the root relaxation log, you may want to try
setting this parameter to 0.

Note: Only affects mixed integer programming (MIP) models