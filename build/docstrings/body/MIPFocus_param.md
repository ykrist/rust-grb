The `MIPFocus` parameter allows you to modify your high-level solution strategy, depending on your goals. By default,
the Gurobi MIP solver strikes a balance between finding new feasible solutions and proving that the current solution is
optimal. If you are more interested in finding feasible solutions quickly, you can select MIPFocus=1. If you believe the
solver is having no trouble finding good quality solutions, and wish to focus more attention on proving optimality,
select MIPFocus=2. If the best objective bound is moving very slowly (or not at all), you may want to try MIPFocus=3 to
focus on the bound.

Note: Only affects mixed integer programming (MIP) models