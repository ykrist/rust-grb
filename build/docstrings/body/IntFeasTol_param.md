An integrality restriction on a variable is considered satisfied when the variable's value is less than `IntFeasTol`
from the nearest integer value. Tightening this tolerance can produce smaller integrality violations, but very tight
tolerances may significantly increase runtime. Loosening this tolerance rarely reduces runtime.

Note: Only affects mixed integer programming (MIP) models