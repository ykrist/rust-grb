A unique identifier used to authenticate an application on a Gurobi Cluster Manager.

You can provide either an access ID and a secret key, or a username and password, to authenticate your connection to a
Cluster Manager.

You must set this parameter through either a gurobi.lic file (using CSAPIACCESSID=YOUR_API_ID) or an empty environment.
Changing the parameter after your environment has been started will result in an error.

Note: Cluster Manager only