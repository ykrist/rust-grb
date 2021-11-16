A target gap to be reached. As soon as the tuner has found parameter settings that allow Gurobi to reach the target gap
for the given model(s), it stops trying to improve parameter settings further. Instead, the tuner switches into the
cleanup phase (see `TuneCleanup` parameter).