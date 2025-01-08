#!/bin/bash

set -euo pipefail

dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

pushd "$dir"

pushd ".."
for f in doc/*.bpmd; do
    {
        cargo run --release -- -i "$f" -o "${f%.bpmd}.xml" && \
        doc/node_modules/.bin/bpmn-to-image "${f%.bpmd}.xml":"${f%.bpmd}.png" && \
        : #rm "${f%.bpmd}.xml"
    } &
done

wait

popd
asciidoctor -o doc.html doc.adoc
