Performance on a MIP model can sometimes experience significant variations due to random effects. As a result, the
tuning tool may return parameter sets that improve on the baseline only due to randomness. This parameter allows you to
perform multiple solves for each parameter set, using different `Seed` values for each, in order to reduce the influence
of randomness on the results. The default value of 0 indicates an automatic choice that depends on model
characteristics.