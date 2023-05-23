/*
 * RUST version to batch updates json documents.
 *
 * The Programe is effectively equal to the shell script:
 * ------------------------------------------------------------------------
 * function put_es_docs {
 *     # $1 may be es doc path, eg, localhost:9200/idx/_doc
 *
 *     while IFS= read -r line
 *     do
 *         id=$(jq --raw-output '._id' <<< "$line")
 *         method=$([[ -z $id ]] && echo POST || echo PUT)
 *         jq '._source' <<< "$line" |
 *             curl-json -X$method "$1/$id" -d @- |
 *             jq -c '[._id, .result]'
 *     done
 * }
 *
 */

use std::str;
use std::env;
use std::process;
use std::io;
use std::io::{BufRead, Read, /*Write*/};
use serde_json::{Value};
//use json_value_remove::Remove;
use curl::easy;

fn basename(s: &str) -> &str {
    match s.rsplit('/').next() {
        Some(name) => name,
        _ => s,
    }
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();
    match &args[..] {
        [_, url] => {
            url.to_string()
        },
        [prog_, .. ] => {
            let prog = basename(prog_);
            println!("usage: {} URL", prog);
            println!("   eg: {} localhost:9200/idx/_doc", prog);
            process::exit(1);
        },
        _ => {
            process::exit(99);
        }
    }
}

fn add_doc(is_new: bool, url: &String, doc: &Value) -> io::Result<()> 
{
    let s = doc.to_string();
    let mut bytes = s.as_bytes();
    //println!("{} {}\n{}\n", if is_new { "POST"} else {"PUT"}, url, s);

    let mut headers = easy::List::new();
    headers.append("Content-Type: application/json")?;

    let mut easy = easy::Easy::new();
    if is_new {
        easy.post(true)?;  //insert
    } else {
        easy.put(true)?;   //update
    }
    easy.url(url)?;
    easy.http_headers(headers)?;
    easy.post_field_size(bytes.len() as u64)?;

    let mut transfer = easy.transfer();
    transfer.read_function(|buf| {
        Ok(bytes.read(buf).unwrap_or(0))
    })?;

    transfer.write_function(|res| {
        //io::stdout().write_all(res)?;
        let res_str = str::from_utf8(&res).unwrap();
        let res_json: Value = serde_json::from_str(&res_str).unwrap();
        println!("{}: {}", res_json["_id"], res_json["result"]);
        Ok(res.len())
    })?;
    transfer.perform()?;

    Ok(())
}

fn main() -> io::Result<()> {
    let id_key = "_id";
    let data_key = "_source";

    let mut url = parse_args();
    if ! url.ends_with("/") {
        url.push_str("/");
    }
    let url_base_len = url.len();

    let mut lines = io::stdin().lock().lines();
    while let Some(line) = lines.next() {
        let data = line?;
        if data.len() == 0 {
            continue;
        }
        let json: Value = serde_json::from_str(&data)?;

        // etract 'id' and append to url, if any
        url.truncate(url_base_len);
        let is_new = if let Value::String(id) = &json[id_key] {
            url.push_str(id);
            false
        } else {
            true
        };

        add_doc(is_new, &url, &json[data_key])?;
    }

    Ok(())
}
