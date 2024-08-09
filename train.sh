#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

source base_venv/bin/activate
pip install -r requirements.txt

python model/training.py -n $SLURM_NPROCS
