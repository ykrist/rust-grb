Frequency of the `RINS` heuristic. Default value (-1) chooses automatically. A value of 0 shuts off RINS. A positive
value n applies `RINS` at every n-th node of the MIP search tree.

Increasing the frequency of the `RINS` heuristic shifts the focus of the MIP search away from proving optimality, and
towards finding good feasible solutions. We recommend that you try MIPFocus, ImproveStartGap, or `ImproveStartTime`
before experimenting with this parameter.

Note: Only affects mixed integer programming (MIP) models