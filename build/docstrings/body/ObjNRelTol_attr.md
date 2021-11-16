This attribute is used to set the allowable degradation for objective $n$ when doing hierarchical multi-objective
optimization for MIP models. You set $n$ using the ObjNumber parameter.

Hierarchical multi-objective MIP optimization will optimize for the different objectives in the model one at a time, in
priority order. If it achieves objective value $z$ when it optimizes for this objective, then subsequent steps are
allowed to degrade this value by at most ObjNRelTol*$\vert z\vert$.

Objective degradations are handled differently for multi-objective LP models. The allowable degradation is controlled
strictly using the ObjNAbsTol.

The default relative tolerance for an objective is 0.

The number of objectives in the model can be queried (or modified) using the `NumObj` attribute.

Please refer to the discussion of Multiple Objectives for more information on the use of alternative objectives.