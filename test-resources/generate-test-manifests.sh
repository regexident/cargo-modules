#!/bin/sh

find "$(pwd)/test-resources/" -maxdepth 1 -mindepth 1 -type d | while read -r dir
do 
    echo "$dir"
    cd "$dir" || exit
    cargo metadata --no-deps --format-version 1 |jq ''> "$dir.json"
    sed "s#$(dirname "$dir")#test-resources#" -i "$dir.json"
    cd - || exit
done
