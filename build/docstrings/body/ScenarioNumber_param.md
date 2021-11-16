When working with multiple scenarios, this parameter selects the index of the scenario you want to work with. When you
query or modify an attribute associated with multiple scenarios (ScenNLB, ScenNUB, ScenNObj, ScenNRHS, etc.), the
`ScenarioNumber` parameter will determine which scenario is actually affected. The value of this parameter should be
less than the value of the NumScenarios attribute (which captures the number of scenarios in the model).

Please refer to the discussion of Multiple Scenarios for more information on the use of alternative scenarios.