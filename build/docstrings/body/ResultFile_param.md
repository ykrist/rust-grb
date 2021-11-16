Specifies the name of the result file to be written upon completion of optimization. The type of the result file is
determined by the file suffix. The most commonly used suffixes are .sol (to capture the solution vector), .bas (to
capture the simplex basis), and .mst (to capture the solution vector on the integer variables). You can also write a
.ilp file (to capture the IIS for an infeasible model), or a .mps, .rew, .lp, or .rlp file (to capture the original
model), or a .dua or .dlp file (to capture the dual of a pure LP model). The file suffix may optionally be followed by
.gz, .bz2, or .7z, which produces a compressed result.

More information on the file formats can be found in the File Format section.