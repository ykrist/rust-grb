This attribute is used to set the allowable degradation for objective $n$ when doing hierarchical multi-objective
optimization. You set $n$ using the ObjNumber parameter.

Hierarchical multi-objective MIP optimization will optimize for the different objectives in the model one at a time, in
priority order. If it achieves objective value $z$ when it optimizes for this objective, then subsequent steps are
allowed to degrade this value by at most ObjNAbsTol.

Objective degradations are handled differently for multi-objective LP models. For LP models, solution quality for
higher-priority objectives is maintained by fixing some variables to their values in previous optimal solutions. These
fixings are decided using variable reduced costs. The value of the `ObjNAbsTol` parameter indicates the amount by which
a fixed variable's reduced cost is allowed to violate dual feasibility. The value of the related `ObjNRelTol` attribute
is ignored.

The default absolute tolerance for an objective is 1e-6.

The number of objectives in the model can be queried (or modified) using the `NumObj` attribute.

Please refer to the discussion of Multiple Objectives for more information on the use of alternative objectives.