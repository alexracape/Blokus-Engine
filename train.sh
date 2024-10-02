#!/bin/bash
#SBATCH --mail-type=BEGIN,END,FAIL

cd /mnt/hpc/tmp/aracape
cp /home/aracape/Blokus-Engine/trainig_requirements.txt .
python -m venv venv
source venv/bin/activate
pip install --upgrade --no-cache-dir blokus-engine
pip install -r training_requirements.txt
cd /home/aracape/Blokus-Engine

python model/training.py --cpus $SLURM_NPROCS
