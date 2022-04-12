#!/bin/bash -i
set -e
ENV_PATH=./env
conda create -y -p $ENV_PATH -c gurobi python==3.8 gurobi==9.5
ENV_PATH_FULL=`readlink -f ${ENV_PATH}`

conda env config vars set -p $ENV_PATH "LD_LIBRARY_PATH=${ENV_PATH_FULL}/lib:${LD_LIBRARY_PATH}"
conda env config vars list -p $ENV_PATH
conda activate ./env
conda config --set env_prompt '(rust-grb)'
echo "Done."
echo "Activate the environment with \`conda activate ${ENV_PATH}\`"