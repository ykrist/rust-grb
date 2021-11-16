The password for connecting to the server (either a Compute Server or a token server).

For connecting to the Remote Services cluster referred to by the `ComputeServer` parameter, you'll need to supply the
client password. Refer to the Gurobi Remote Services Reference Manual for more information on starting Compute Server
jobs.

Supply the token server password (if needed) when connecting to the server referred to by the `TokenServer` parameter,

You must set this parameter through either a gurobi.lic file (using PASSWORD=pwd) or an empty environment. Changing the
parameter after your environment has been created will have no effect.