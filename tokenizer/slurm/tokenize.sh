#!/bin/bash
#SBATCH --job-name=tokenize_morph
#SBATCH --output=logs/tokenize_%j.out
#SBATCH --error=logs/tokenize_%j.err
#SBATCH --partition=<your_cpu_partition>
#SBATCH --time=12:00:00
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --nodes=1
#SBATCH --ntasks=1

# IMPORTANT: Edit the --partition value above to match your cluster.
# Run `sinfo` to see available partitions.

set -e

echo "Job started: $(date)"
echo "Node: $(hostname)"
echo "Job ID: $SLURM_JOB_ID"

# Set up environment
module load python/3.11
source $HOME/.cargo/env
cd <your-working-directory>
source .venv/bin/activate
export PATH="$HOME/.cargo/bin:$PATH"

# Sanity check that environment is correct
python -c "from turkish_tokenizer import TurkishTokenizer; print('TurkishTokenizer ok')"

# Run with unbuffered output (-u) so logs update in real time
python -u scripts/tokenize_morph.py

echo "Job finished: $(date)"