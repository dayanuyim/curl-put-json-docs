#!/bin/bash


 function put_doc {
     while IFS= read -r line
     do
         id=$(jq --raw-output '._id' <<< "$line")
         method=$([[ -z $id ]] && echo POST || echo PUT)
         jq '._source' <<< "$line" |
             curl -s -H 'Content-Type: application/json' -X$method "$1/$id" -d @- |
             jq -c '[._id, .result]'
     done
 }

# $1 is url, eg, localhost:9200/idx/_doc
if [ -z "$1" ]; then
    echo "usage: ${0##*/} URL" >&2
    exit 1
fi
 put_doc "$1"
