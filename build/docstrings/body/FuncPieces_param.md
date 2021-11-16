This parameter sets the strategy used for performing a piecewise-linear approximation of a function constraint. There
are a few options:

`FuncPieces` >= 2: Sets the number of pieces; pieces are equal width.

`FuncPieces` = 1: Uses a fixed width for each piece; the actual width is provided in the `FuncPieceLength` parameter.

`FuncPieces` = -1: Bounds the absolute error of the approximation; the error bound is provided in the `FuncPieceError`
parameter.

`FuncPieces` = -2: Bounds the relative error of the approximation; the error bound is provided in the `FuncPieceError`
parameter.

This parameter only applies to function constraints whose `FuncPieces` attribute has been set to $0$.

See the discussion of function constraints for more information.