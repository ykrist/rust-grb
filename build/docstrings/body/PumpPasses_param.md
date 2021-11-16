Number of passes of the feasibility pump heuristic. Note that this heuristic is only applied at the end of the MIP root.

This heuristic is quite expensive, and generally produces poor quality solutions. You should generally only use it if
other means, including exploration of the tree with default settings, fail to produce a feasible solution.

Note: Only affects mixed integer programming (MIP) models