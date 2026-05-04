#!/bin/bash
#SBATCH --job-name=download_sample
#SBATCH --output=<your-working-directory>/logs/download_sample_%j.out>
#SBATCH --error=<your-working-directory>/logs/download_sample_%j.err>
#SBATCH --partition=<your_cpu_partition>
#SBATCH --qos=<your_qos>
#SBATCH --time=7-00:00:00
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --mem=16G

module load python/3.11
cd <your-working-directory>
source venv/bin/activate
pip install datasets huggingface_hub tqdm requests -q
python download_sample.py