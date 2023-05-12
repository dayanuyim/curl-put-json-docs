/*
 * RUST version to batch updates json documents.
 *
 * The Programe is effectively equal to the shell script:
 * ------------------------------------------------------------------------
 * function put_doc {
 *   es_idx="$1"
 *   
 *   while IFS= read -r line
 *   do
 *     id=$(jq --raw-output '.id' <<< $line)
 *     jq 'del(.id)' <<< $line |
 *       curl -s -H 'Content-Type: application/json' -XPUT "$es_idx/_doc/$id" -d @- |
 *       jq -c '[._id, .result]'
 *   done
 * }
 *
 */

use std::str;
use std::env;
use std::process;
use std::io;
use std::io::{BufRead, Read, /*Write*/};
use serde_json::{Value};
use json_value_remove::Remove;
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
            println!("   eg: {} localhost:9200/cve/_doc", prog);
            process::exit(1);
        },
        _ => {
            process::exit(99);
        }
    }
}

fn put_doc(url: &String, doc: &String) -> io::Result<()> 
{
    let mut bytes = doc.as_bytes();

    let mut headers = easy::List::new();
    headers.append("Content-Type: application/json")?;

    let mut easy = easy::Easy::new();
    easy.url(url)?;
    easy.http_headers(headers)?;
    easy.put(true)?;
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
    let mut url = parse_args();
    let url_base_len = url.len();

    let mut lines = io::stdin().lock().lines();
    while let Some(line) = lines.next() {
        let mut data = line?;
        if data.len() == 0 {
            continue;
        }

        url.truncate(url_base_len);

        // etract 'id' and append to url, if any
        let mut json: Value = serde_json::from_str(&data)?;
        if let Some(Value::String(id)) = json.remove("/id")? {
            url.push_str("/");
            url.push_str(&id);
            data = json.to_string();  //data without id
        };

        put_doc(&url, &data)?;
    }

    Ok(())
}
