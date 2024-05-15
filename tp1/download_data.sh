#/bin/sh

mkdir data

echo "Clone"
git clone https://huggingface.co/datasets/flax-sentence-embeddings/stackexchange_title_body_jsonl data

cd data
# Este archivo es el mas grande, solo agrega complejidad logistica
rm stackoverflow.com-Posts.jsonl.gz

for filename in *.gz; do
    curl -o $filename -L https://huggingface.co/datasets/flax-sentence-embeddings/stackexchange_title_body_jsonl/resolve/main/$filename 
    gunzip -v $filename
done