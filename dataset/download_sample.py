"""
download_sample.py
Downloads a random 100GB sample from tascib/turkish-llm-dataset.
Randomly selects shards instead of reading sequentially.

Usage:
    python download_sample.py
"""

import json
import os
import random
import requests

from huggingface_hub import HfApi

OUTPUT_DIR    = "/cta/users/fastinference1/turkish_dataset/sample_data" # CHANGE to your desired output directory
OUTPUT_FILE   = os.path.join(OUTPUT_DIR, "sample_100gb.jsonl")
TARGET_RECORDS = 50_000_000
REPO_ID       = "tascib/turkish-llm-dataset"
SHARD_SIZE    = 50_000   # records per shard

os.makedirs(OUTPUT_DIR, exist_ok=True)

# ===============================================================
# Get all shard file paths from HuggingFace
# ===============================================================

print("Fetching shard list from HuggingFace...")
api = HfApi()
all_files = api.list_repo_files(REPO_ID, repo_type="dataset")
shard_files = sorted([f for f in all_files if f.startswith("data/dataset_part_") and f.endswith(".jsonl")])
print(f"Total shards available: {len(shard_files)}")

# Randomly shuffle shards
random.seed(42)
random.shuffle(shard_files)

# ===============================================================
# Download shards until TARGET_RECORDS is reached
# ===============================================================

count       = 0
shards_used = 0
base_url    = f"https://huggingface.co/datasets/{REPO_ID}/resolve/main/"

with open(OUTPUT_FILE, "w", encoding="utf-8") as out_f:
    for shard_path in shard_files:
        if count >= TARGET_RECORDS:
            break

        url = base_url + shard_path
        print(f"\nDownloading {shard_path}...")

        try:
            response = requests.get(url, stream=True, timeout=120)
            response.raise_for_status()

            for line in response.iter_lines():
                if not line:
                    continue
                try:
                    record = json.loads(line)
                    text = record.get("text", "")
                    if text:
                        out_f.write(json.dumps({"text": text}, ensure_ascii=False) + "\n")
                        count += 1
                except json.JSONDecodeError:
                    continue

            shards_used += 1
            size_gb = os.path.getsize(OUTPUT_FILE) / 1e9
            print(f"  Progress: {count:,} records, {size_gb:.2f} GB  (shard {shards_used}/{len(shard_files)})")

        except Exception as e:
            print(f"  Failed to download {shard_path}: {e}, skipping...")
            continue

size_gb = os.path.getsize(OUTPUT_FILE) / 1e9
print(f"\nDone.")
print(f"Total records : {count:,}")
print(f"Total size    : {size_gb:.2f} GB")
print(f"Shards used   : {shards_used}")
print(f"Output file   : {OUTPUT_FILE}")