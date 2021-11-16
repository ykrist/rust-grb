When the model has multiple scenarios, this attribute is used to query the objective value of the current solution for
scenario $n$. You set $n$ using the ScenarioNumber parameter. If no solution is available, this returns GRB_INFINITY
(for a minimization objective).

The number of scenarios in the model can be queried (or modified) using the `NumScenarios` attribute.

Please refer to the Multiple Scenarios discussion for more information.