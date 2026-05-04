# Turkish Morphological Tokenizer — Setup & Tokenization Guide

Complete guide to set up the TurkishTokenizer on a fresh HPC cluster and tokenize a
Turkish JSONL corpus into binary token files.

### THIS TOKENIZER'S VOCAB SIZE IS 32769 NOT 32768

## What you'll have at the end

- A working Python environment with TurkishTokenizer installed
- Two binary files: `train_morph.bin` and `val_morph.bin` containing tokenized text
  (uint16 token IDs) ready to be loaded by training code

## Prerequisites on the cluster

- Python 3.11+ (via module load or system install)
- Free disk space
- Internet access from compute or login nodes (for pip and rustup downloads)

## Step 1 — If you can, get on a compute node. It's OK if otherwise.

Don't install on the login node. Get an interactive session:

```bash
# Adjust partition name and resources for your cluster
srun --pty --partition=<your_cpu_partition> --time=2:00:00 --cpus-per-task=4 --mem=16G bash
```

If you don't know your partition names, run `sinfo` to list them.


## Step 3 — Load Python module

Check what's available:

```bash
module avail python
```

Load Python 3.11 (adjust name to match your cluster):

```bash
module load python/3.11
```

Verify:

```bash
python3.11 --version
which python3.11
```

## Step 4 — Install Rust toolchain

The TurkishTokenizer compiles from Rust source. It needs **Rust 1.85+** (for edition 2024 features).

### Check if cluster has a recent enough Rust module

```bash
module avail rust
```

If you see Rust 1.85 or newer, use it:

```bash
module load rust/<version>
cargo --version  # must show 1.85+
```

### If cluster Rust is too old (most common case), install via rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When prompted, hit Enter (option 1) for default install. Installs to `~/.cargo/`.

Activate it in current shell:

```bash
source $HOME/.cargo/env
cargo --version  # should show recent stable Rust
```

Make persistent across sessions:

```bash
echo 'source $HOME/.cargo/env' >> ~/.bashrc
```

## Step 5 — Create/Activate Python virtual environment

The venv must use Python 3.11 explicitly to avoid version mismatches:

```bash
python3.11 -m venv .venv
source .venv/bin/activate

# Verify
which python   # should be inside .venv/bin/
python --version  # should be 3.11.x
```

## Step 6 — Fix PATH so cargo is visible in the venv

Activating a venv can shadow `cargo` in PATH. After activating venv, run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
which cargo  # must show ~/.cargo/bin/cargo
```

## Step 7 — Install Python dependencies

```bash
pip install --upgrade pip setuptools wheel
pip install numpy tqdm
```

That's it for tokenization. No torch needed.

## Step 8 — Install TurkishTokenizer

```bash
# Clean any prior build artifacts
cd turkish_morph_tokenizer && cargo clean && cd ..
pip cache purge

# Install (compiles Rust — takes 2–5 min first time)
pip install ./turkish_morph_tokenizer
```

Watch for errors. Common issue: if you see `feature 'edition2024' is required`, your Rust is too old. Go back to Step 4 and install rustup.

## Step 9 — Verify TurkishTokenizer works

```bash
python -c "from turkish_tokenizer import TurkishTokenizer; t = TurkishTokenizer(); print(t.encode('Merhaba dünya'))"
```

Should print a list of integer token IDs, e.g. `[2, 0, 1234, 5678, ...]`.

If this works, your environment is ready.

## Step 11 — Place your corpus

Your corpus should be a JSONL file where each line is a JSON object with a `text` field:

```json
{"text": "First Turkish document content..."}
{"text": "Second Turkish document content..."}
```

Place it at `data/corpus.jsonl` (or update the path in `scripts/tokenize_morph.py`).

## Step 12 — Test on a small sample first

Always test before running on the full corpus:

```bash
# Make a 1000-line sample
head -n 1000 data/corpus.jsonl > data/test_1k.jsonl

# Temporarily edit scripts/tokenize_morph.py:
#   INPUT_JSONL = "data/test_1k.jsonl"

# Run
python scripts/tokenize_morph.py
```

Should finish in under a minute. Verify both `.bin` files exist with non-zero size.

Then revert `INPUT_JSONL` back to your real corpus path.

## Step 13 — Run full tokenization

```bash
mkdir -p logs
sbatch slurm/tokenize.sh

# Monitor
squeue -u $USER
tail -f logs/tokenize_*.out
```

Edit `slurm/tokenize.sh` first to set the correct partition name for your cluster.

## Step 14 — Validate the output

```bash
python scripts/validate_morph.py
```

Look for:
- Both files exist with non-zero size
- Max ID ≤ 32768 (within vocab)
- Decoded text is recognizable Turkish
- Reasonable unique token count (thousands)

## Final directory layout

```
turkish_tokenizer_setup/
├── README.md
├── .gitignore
├── turkish_morph_tokenizer/          # local Rust tokenizer source
├── scripts/
│   ├── tokenize_morph.py
│   └── validate_morph.py
├── slurm/
│   └── tokenize.sh
└── logs/                             # gitignored
```

## Common issues

**"feature `edition2024` is required" during pip install.**
Rust is too old. Cluster modules often lag behind. Use rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**"cargo: command not found" after activating venv.**
Venv shadows PATH. After activation:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

**Python imports fail despite `pip list` showing the package.**
Mixed Python versions in the venv. Rebuild cleanly:
```bash
deactivate
rm -rf .venv
module load python/3.11
python3.11 -m venv .venv
source .venv/bin/activate
# reinstall everything
```

**Slurm log file not updating live.**
Python output buffering. The provided `slurm/tokenize.sh` already uses `python -u`.

**OOM during tokenization.**
Shouldn't happen with the streaming script — memory is constant. If it does, reduce `MAX_TOKENS` in `scripts/tokenize_morph.py` to a tighter estimate (e.g., 8B for a 10GB corpus).

## Quick command reference

```bash
# Test tokenizer
python -c "from turkish_tokenizer import TurkishTokenizer; t=TurkishTokenizer(); print(t.encode('test'))"

# Run tokenization (small test)
python scripts/tokenize_morph.py

# Submit full tokenization job
sbatch slurm/tokenize.sh

# Monitor
squeue -u $USER
tail -f logs/tokenize_*.out

# Validate output
python scripts/validate_morph.py
```