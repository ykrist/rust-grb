The router node for a Remote Services cluster. A router can be used to improve the robustness of a Compute Server
deployment. You can refer to the router using either its name or its IP address. A typical Remote Services deployment
won't use a router, so you typically won't need to set this parameter.

Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server jobs.

You must set this parameter through either a gurobi.lic file (using ROUTER=name) or an empty environment. Changing the
parameter after your environment has been created will have no effect.