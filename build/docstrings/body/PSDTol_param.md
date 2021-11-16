Sets a limit on the amount of diagonal perturbation that the optimizer is allowed to perform on a Q matrix in order to
correct minor PSD violations. If a larger perturbation is required, the optimizer will terminate with a
GRB_ERROR_Q_NOT_PSD error.

Note: Only affects QP/QCP/MIQP/MIQCP models