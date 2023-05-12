#!/bin/bash


 function put_doc {
   es_idx="$1"
   
   while IFS= read -r line
   do
     id=$(jq --raw-output '.id' <<< $line)
     jq 'del(.id)' <<< $line |
       curl -s -H 'Content-Type: application/json' -XPUT "$es_idx/_doc/$id" -d @- |
       jq -c '[._id, .result]'
   done
 }

put_doc "$1"
