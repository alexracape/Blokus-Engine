#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

python -m venv venv
source venv/bin/activate
pip install -r model/requirements.txt

cd self_play
maturin develop
cd ..s

python model/training.py -n $SLURM_NPROCS
