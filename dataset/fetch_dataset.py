from datasets import load_dataset
import json

# load the first 10,000 articles from English Wikipedia
dataset = load_dataset("wikimedia/wikipedia", "20231101.en", split="train", streaming=True)

records = []
for i, article in enumerate(dataset):
    if i >= 10000:
        break
    records.append({
        "key": article["title"],
        "value": article["text"][:500]  # first 500 chars of the article
    })

with open("dataset/wikipedia_10k.json", "w") as f:
    json.dump(records, f)

print(f"Saved {len(records)} articles to dataset/wikipedia_10k.json")