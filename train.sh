#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

python model/training.py -n $SLURM_NPROCS
