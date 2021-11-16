This parameter limits the number of branch-and-bound nodes explored when completing a partial MIP start. The default
value of -1 uses the value of the `SubMIPNodes` parameter. A value of -2 means to only check full MIP starts for
feasibility and to ignore partial MIP starts. A value of -3 shuts off MIP start processing entirely. Non-negative values
are node limits.

Note: Only affects mixed integer programming (MIP) models