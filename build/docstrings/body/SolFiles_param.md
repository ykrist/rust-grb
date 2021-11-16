During the MIP solution process, multiple incumbent solutions are typically found on the path to finding a proven
optimal solution. Setting this parameter to a non-empty string causes these solutions to be written to files (in .sol
format) as they are found. The MIP solver will append _n.sol to the value of the parameter to form the name of the file
that contains solution number $n$. For example, setting the parameter to value solutions/mymodel will create files
mymodel_0.sol, mymodel_1.sol, etc., in directory solutions.

Note that intermediate solutions can be retrieved as they are generated through a callback (by requesting the MIPSOL_SOL
in a MIPSOL callback). This parameter makes the process simpler.

Note: Only affects mixed integer programming (MIP) models