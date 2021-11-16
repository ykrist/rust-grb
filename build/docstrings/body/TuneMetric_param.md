A single tuning run typically produces multiple timing results for each candidate parameter set, either as a result of
performing multiple trials, or tuning multiple models, or both. This parameter controls how these results are aggregated
into a single measure. The default setting (-1) chooses the aggregation automatically; setting 0 computes the average of
all individual results; setting 1 takes the maximum.