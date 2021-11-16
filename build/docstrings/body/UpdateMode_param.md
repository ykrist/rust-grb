Determines how newly added variables and linear constraints are handled. The default setting (1) allows you to use new
variables and constraints immediately for building or modifying the model. A setting of 0 requires you to call update
before these can be used.

Since the vast majority of programs never query Gurobi for details about the optimization models they build, the default
setting typically removes the need to call update, or even be aware of the details of our lazy update approach for
handling model modifications. However, these details will show through when you try to query modified model information.

In the Gurobi interface, model modifications (bound changes, right-hand side changes, objective changes, etc.) are
placed in a queue. These queued modifications are applied to the model at three times: when you call update, when you
call optimize, or when you call write to write the model to disk. When you query information about the model, the result
will depend on both whether that information was modified and when it was modified. In particular, no matter what
setting of `UpdateMode` you use, if the modification is sitting in the queue, you'll get the result from before the
modification.

To expand on this a bit, all attribute modifications are actually placed in a queue. This includes attributes that may
not traditionally be viewed as being part of the model, including things like variable branching priorities, constraint
basis statuses, etc. Querying the values of these attributes will return their previous values if subsequent
modifications are still in the queue.

The only potential benefit to changing the parameter to 0 is that in unusual cases this setting may allow simplex to
make more aggressive use of warm-start information after a model modification.

If you want to change this parameter, you need to set it as soon as you create your Gurobi environment.

Note that you still need to call update to modify an attribute on an SOS constraint, quadratic constraint, or general
constraint.