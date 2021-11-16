The MIP solver can change parameter settings in the middle of the search in order to adopt a strategy that gives up on
moving the best bound and instead devotes all of its effort towards finding better feasible solutions. This parameter
allows you to specify the node count at which the MIP solver switches to a solution improvement strategy. For example,
setting this parameter to 10 will cause the MIP solver to switch strategies once the node count is larger than 10.

Note: Only affects mixed integer programming (MIP) models