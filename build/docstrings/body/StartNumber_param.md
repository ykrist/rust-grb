This parameter selects the index of the MIP start you want to work with. When you modify a MIP start value (using the
Start attribute) the `StartNumber` parameter will determine which MIP start is actually affected. The value of this
parameter should be less than the value of the NumStart attribute (which captures the number of MIP starts in the
model).

The special value -1 is meant to append new MIP start to a model, but querying a MIP start when `StartNumber` is -1 will
result in an error.

Note: Only affects mixed integer programming (MIP) models