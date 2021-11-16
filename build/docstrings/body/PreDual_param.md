Controls whether presolve forms the dual of a continuous model. Depending on the structure of the model, solving the
dual can reduce overall solution time. The default setting uses a heuristic to decide. Setting 0 forbids presolve from
forming the dual, while setting 1 forces it to take the dual. Setting 2 employs a more expensive heuristic that forms
both the presolved primal and dual models (on two threads), and heuristically chooses one of them.

Note: LP only