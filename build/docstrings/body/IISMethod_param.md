Chooses the IIS method to use. `Method` 0 is often faster, while method 1 can produce a smaller IIS. `Method` 2 ignores
the bound constraints. `Method` 3 will return the IIS for the LP relaxation of a MIP model if the relaxation is
infeasible, even though the result may not be minimal when integrality constraints are included. The default value of -1
chooses automatically.