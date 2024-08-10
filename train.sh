#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

cd /mnt/hpc/tmp/aracape
cp /home/aracape/Blokus-Engine/requirements.txt .
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
cd /home/aracape/Blokus-Engine

python model/training.py -n $SLURM_NPROCS
