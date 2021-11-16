A target runtime in seconds to be reached. As soon as the tuner has found parameter settings that allow Gurobi to solve
the model(s) within the target runtime, it stops trying to improve parameter settings further. Instead, the tuner
switches into the cleanup phase (see `TuneCleanup` parameter).