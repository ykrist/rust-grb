Variable branching priority. The value of this attribute is used as the primary criterion for selecting a fractional
variable for branching during the MIP search. Variables with larger values always take priority over those with smaller
values. Ties are broken using the standard branch variable selection criteria. The default variable branch priority
value is zero.

Note that deleting variables from your model will cause several attributes to be discarded (variable hints and branch
priorities). If you'd like them to persist, your program will need to repopulate them after deleting the variables and
making a subsequent model update call.

Only affects MIP models.