When set to 1, enable the local creation of models, and later submit batch-optimization jobs to the Cluster Manager. See
the Batch Optimization section for more details. Note that if `CSBatchMode` is enabled, only batch-optimization calls
are allowed.

You must set this parameter through either a gurobi.lic file (using CSBATCHMODE=1) or an empty environment. Changing the
parameter after your environment has been started will result in an error.

Note: Cluster Manager only