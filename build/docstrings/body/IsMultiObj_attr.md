Indicates whether the model has multiple objectives.

Note that the case where the model has a single objective (NumObj = 1) is slightly ambiguous. If you used setObjectiveN
to set your objective, or if you set any of the multi-objective attributes (e.g., ObjNPriority), then the model is
considered to be a multi-objective model. Otherwise, it is not.

To reset a multi-objective model back to a single objective model, you should set the `NumObj` attribute to 0, call
model update, and then set a new single objective.