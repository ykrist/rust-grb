Enables or disables sifting within dual simplex. `Sifting` can be useful for LP models where the number of variables is
many times larger than the number of constraints (we typically only see significant benefits when the ratio is 100 or
more). Options are Automatic (-1), Off (0), Moderate (1), and Aggressive (2). With a Moderate setting, sifting will be
applied to LP models and to the initial root relaxation for MIP models. With an Aggressive setting, sifting will be
applied any time dual simplex is used, including at the nodes of a MIP. Note that this parameter has no effect if you
aren't using dual simplex. Note also that Gurobi will ignore this parameter in cases where sifting is obviously a worse
choice than dual simplex.