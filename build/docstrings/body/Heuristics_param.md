Determines the amount of time spent in MIP heuristics. You can think of the value as the desired fraction of total MIP
runtime devoted to heuristics (so by default, we aim to spend 5% of runtime on heuristics). Larger values produce more
and better feasible solutions, at a cost of slower progress in the best bound.

Note: Only affects mixed integer programming (MIP) models