Shuts off a few reductions in order to allow presolve to transform any constraint on the original model into an
equivalent constraint on the presolved model. You should consider setting this parameter to 1 if you are using callbacks
to add your own cuts. A cut that cannot be applied to the presolved model will be silently ignored. The impact on the
size of the presolved problem is usually small.