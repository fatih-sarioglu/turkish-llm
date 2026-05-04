"""Sanity check the tokenized .bin files.

Reads samples from train_morph.bin and val_morph.bin and verifies:
- Files exist and have non-zero size
- Token IDs are within vocabulary range
- Decoded text looks like Turkish
"""
import os
import numpy as np
from turkish_tokenizer import TurkishTokenizer

OUT_DIR = "data/tokenized"
VOCAB_SIZE = 32769  # TurkishTokenizer max ID + 1

morph = TurkishTokenizer()


def check_bin(path):
    print(f"\n{'='*60}\nChecking: {path}\n{'='*60}")
    if not os.path.exists(path):
        print("  ERROR: file does not exist!")
        return

    arr = np.memmap(path, dtype=np.uint16, mode='r')
    print(f"  Tokens: {len(arr):,}")
    print(f"  Size:   {arr.nbytes / 1e9:.2f} GB")

    sample = arr[:100_000] if len(arr) > 100_000 else arr[:]
    print(f"  Min ID: {sample.min()}")
    print(f"  Max ID: {sample.max()}")
    print(f"  Vocab limit: {VOCAB_SIZE}")
    if sample.max() >= VOCAB_SIZE:
        print(f"  WARNING: max ID >= vocab size!")

    n_unique = len(np.unique(sample))
    print(f"  Unique tokens in sample: {n_unique:,}")

    # Check distribution — too many zeros suggests a problem
    n_zeros = (sample == 0).sum()
    pct_zeros = 100 * n_zeros / len(sample)
    print(f"  Zeros in sample: {n_zeros:,} ({pct_zeros:.2f}%)")
    # Note: ID 0 is <uppercase>, so 5-15% is normal for Turkish text

    print(f"\n  First 100 tokens decoded:")
    try:
        decoded = morph.decode(arr[:100].tolist())
        print(f"    {decoded[:400]}")
    except Exception as e:
        print(f"    Decode error: {e}")

    if len(arr) > 1_000_000:
        print(f"\n  100 tokens from middle (offset {len(arr) // 2}):")
        try:
            mid = arr[len(arr) // 2 : len(arr) // 2 + 100].tolist()
            decoded = morph.decode(mid)
            print(f"    {decoded[:400]}")
        except Exception as e:
            print(f"    Decode error: {e}")


check_bin(f"{OUT_DIR}/train_morph.bin")
check_bin(f"{OUT_DIR}/val_morph.bin")

# Summary
train_path = f"{OUT_DIR}/train_morph.bin"
val_path = f"{OUT_DIR}/val_morph.bin"
if os.path.exists(train_path) and os.path.exists(val_path):
    train = np.memmap(train_path, dtype=np.uint16, mode='r')
    val = np.memmap(val_path, dtype=np.uint16, mode='r')
    total = len(train) + len(val)
    val_pct = 100 * len(val) / total if total > 0 else 0
    print(f"\n{'='*60}")
    print(f"Summary")
    print(f"{'='*60}")
    print(f"Total tokens: {total:,}")
    print(f"Val split:    {val_pct:.3f}% (expected ~0.1%)")