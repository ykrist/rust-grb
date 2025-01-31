This attribute sets the strategy used for performing a piecewise-linear approximation of a function constraint. There
are a few options:

`FuncPieces` = 0: Ignore the attribute settings for this function constraint and use the parameter settings (FuncPieces,
etc.) instead.

`FuncPieces` >= 2: Sets the number of pieces; pieces are equal width.

`FuncPieces` = 1: Uses a fixed width for each piece; the actual width is provided in the `FuncPieceLength` attribute.

`FuncPieces` = -1: Bounds the absolute error of the approximation; the error bound is provided in the `FuncPieceError`
attribute.

`FuncPieces` = -2: Bounds the relative error of the approximation; the error bound is provided in the `FuncPieceError`
attribute.

See the discussion of function constraints for more information.