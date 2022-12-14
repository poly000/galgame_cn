use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::Proxy;

use regex::Regex;

const MAX_RETRY: i32 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        sites,
        output,
        proxy,
    } = Args::parse();

    let current_list = unsafe { String::from_utf8_unchecked(fs::read(&output)?) };
    let sites_list = unsafe { String::from_utf8_unchecked(fs::read(sites)?) };
    let mut output = OpenOptions::new().append(true).read(false).open(output)?;

    let client = if let Some(proxy) = proxy {
        Client::builder()
        .proxy(Proxy::all(proxy)?)
    } else {
        Client::builder()
    }
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0 Safari/537.36")
        .build()?;

    let regex = Regex::new("<title[^>]*>(.*)</title>")?;

    for site in sites_list
        .lines()
        .filter(|&site| !current_list.contains(site))
    {
        if site.is_empty() {
            continue;
        } // ignore empty lines

        let mut content = None;
        for _ in 0..MAX_RETRY {
            if let Ok(resp) = client.get(site).send() {
                content = Some(resp.text()?);
                break;
            }
        }

        if let Some(content) = content {
            if let Some(title) = regex.captures(&content).and_then(|cap| cap.get(1)) {
                output.write_fmt(format_args!(
                    "[{}]({site})\n",
                    title
                        .as_str()
                        .trim()
                        .replace("_", "\\_")
                        .replace("*", "\\*")
                ))?
            } else {
                eprintln!("{} does not contains title!", site);
                output.write_fmt(format_args!("[无标题]({site})"))?;
                continue;
            }
        }
    }

    Ok(())
}

use clap::Parser;
use clap::ValueHint;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// plain text file, contains sites to generate
    #[clap(short, long, value_parser, value_hint = ValueHint::FilePath)]
    sites: PathBuf,

    /// result file, should exists
    #[clap(short, long, value_parser, value_hint = ValueHint::FilePath)]
    output: PathBuf,

    /// [optional] proxy to set, for example http://127.0.0.1:8080
    #[clap(short, long, value_parser)]
    proxy: Option<String>,
}
