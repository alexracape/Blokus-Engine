#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

python -m venv venv
source venv/bin/activate
pip install -r requirements.txt

python model/training.py -n $SLURM_NPROCS
