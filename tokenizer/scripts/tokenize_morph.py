"""Tokenize Turkish JSONL corpus with TurkishTokenizer.

Streams to memmap files for low memory usage.
Outputs train_morph.bin and val_morph.bin (uint16 token IDs)
into data/tokenized/.

Input format: JSONL with a "text" field per line.
"""
import os
import json
import numpy as np
from tqdm import tqdm
from turkish_tokenizer import TurkishTokenizer

# === Config ===
INPUT_JSONL = "data/corpus.jsonl"           # CHANGE if needed
OUT_DIR = "data/tokenized"                  # CHANGE if needed
VAL_FRACTION = 0.001                        # 0.1% for validation

# Generous upper bound — file truncated to actual size at end.
# For a 10GB corpus, ~7-8B tokens is typical for the morph tokenizer.
MAX_TOKENS = 200_000_000_000

# TurkishTokenizer EOS token ID (verified by inspecting tokenizer vocab)
MORPH_EOS_ID = 6

os.makedirs(OUT_DIR, exist_ok=True)

# === Load tokenizer ===
print("Loading TurkishTokenizer...", flush=True)
morph = TurkishTokenizer()
print(f"EOS id: {MORPH_EOS_ID}", flush=True)

# === Pass 1: count documents ===
print("\nCounting documents...", flush=True)
n_docs = 0
with open(INPUT_JSONL, "r", encoding="utf-8") as f:
    for _ in f:
        n_docs += 1
print(f"Total documents: {n_docs}", flush=True)

n_val = max(1, int(n_docs * VAL_FRACTION))
n_train = n_docs - n_val
print(f"Train: {n_train}, Val: {n_val}", flush=True)

# === Pre-allocate output files ===
def make_memmap(name, max_tokens):
    path = os.path.join(OUT_DIR, name)
    return path, np.memmap(path, dtype=np.uint16, mode='w+', shape=(max_tokens,))

train_path, train_mm = make_memmap("train_morph.bin", MAX_TOKENS)
val_path,   val_mm   = make_memmap("val_morph.bin",   MAX_TOKENS // 100)

train_pos = 0
val_pos = 0

# === Pass 2: tokenize and stream-write ===
print("\nTokenizing... (slow part)", flush=True)
with open(INPUT_JSONL, "r", encoding="utf-8") as f:
    for i, line in enumerate(tqdm(f, total=n_docs)):
        try:
            doc = json.loads(line)
            text = doc.get("text", "").strip()
        except json.JSONDecodeError:
            continue
        if not text:
            continue

        morph_ids = morph.encode(text)
        morph_ids.append(MORPH_EOS_ID)
        arr = np.array(morph_ids, dtype=np.uint16)

        if i >= n_train:
            val_mm[val_pos:val_pos + len(arr)] = arr
            val_pos += len(arr)
        else:
            train_mm[train_pos:train_pos + len(arr)] = arr
            train_pos += len(arr)

# === Finalize: truncate files to actual size ===
def finalize(path, mm, pos):
    mm.flush()
    del mm
    with open(path, 'r+b') as f:
        f.truncate(pos * 2)  # uint16 = 2 bytes per token
    size_gb = (pos * 2) / 1e9
    print(f"  {path}: {pos:,} tokens ({size_gb:.2f} GB)", flush=True)

print("\nFinalizing files...", flush=True)
finalize(train_path, train_mm, train_pos)
finalize(val_path,   val_mm,   val_pos)

print("\nDone.", flush=True)
print(f"Total tokens: {train_pos + val_pos:,}")