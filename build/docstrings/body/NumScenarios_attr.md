Number of scenarios in the model. Modifying this attribute changes the number: decreasing it discards existing
scenarios; increasing it creates new scenarios (initialized to have no changes w.r.t. the base model); setting it to 0
discards all scenarios so that the base model is no longer a multi-scenario model.

You can use the ScenarioNumber parameter, in conjunction with multi-scenario attributes (ScenNLB, ScenNUB, ScenNObj,
ScenNRHS, ScenNName, etc.), to query or modify attributes for different scenarios. The value of ScenarioNumber should
always be less than NumScenarios.

Please refer to the Multiple Scenarios discussion for more information.