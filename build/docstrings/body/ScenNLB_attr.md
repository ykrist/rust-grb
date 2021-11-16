When the model has multiple scenarios, this attribute is used to query or modify changes of the variable lower bounds in
scenario $n$ w.r.t. the base model. You set $n$ using the ScenarioNumber parameter.

If an element of this array attribute is set to the undefined value (GRB_UNDEFINED in C and C++, or GRB.UNDEFINED in
Java, .NET, and Python), it means that the corresponding value in the scenario is identical to the one in the base
model.

The number of scenarios in the model can be queried (or modified) using the `NumScenarios` attribute.

Please refer to the Multiple Scenarios discussion for more information.