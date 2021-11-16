Limits the total amount of memory (in GB, i.e., $10^9$ bytes) available to Gurobi. If more is needed, Gurobi will fail
with an OUT_OF_MEMORY error (see the Error Code section for further details).

This parameter must be set when the Gurobi environment is first created. You will need to create an empty environment,
set the parameter, and then start the environment.