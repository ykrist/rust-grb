This parameter affects how Gurobi deals with names. If set to 1, subsequent calls to add variables or constraints to the
model will ignore the associated names. Names for objectives and the model will also be ignored. In addition, subsequent
calls to modify name attributes will have no effect. Note that variables or constraints that had names at the point this
parameter was changed to 1 will retain their names. If you wish to discard all name information, you should set this
parameter to 1 before adding variables or constraints to the model.