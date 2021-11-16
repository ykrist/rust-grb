Number of objectives in the model. If you modify this attribute, it will change the number of objectives in the model.
Decreasing it will discard existing objectives. Increasing it will create new objectives (initialized to 0). Setting it
to 0 will create a model with no objective (i.e., a feasibility model). If you want to switch from a multi-objective
model to a single-objective model you also need to set `NumObj` to 0 and call model update before installing a new
single objective.

You can use the ObjNumber parameter, in conjunction with multi-objective attributes (ObjN, ObjNName, etc.), to query or
modify attributes for different objectives. The value of ObjNumber should always be less than NumObj.

Please refer to the discussion of Multiple Objectives for more information on the use of alternative objectives.