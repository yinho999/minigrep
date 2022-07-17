use std::result::Result::Ok;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Error;
use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[clap(author,version,about,long_about=None)]
struct Args {
    /// Target path
    #[clap(value_parser, short, long)]
    path: String,

    /// Regex for search
    #[clap(value_parser)]
    regex: String,
}
#[derive(Debug)]
struct Match {
    line_num: usize,
    line: String,
    regex: Regex,
    file_name: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let path = Path::new(&args.path);
    let regex = Regex::new(&args.regex)?;

    let result = search_dir(
        path,
        &regex,
        &|p, result| {
            for i in result {
                println!("{:?} , {:?}", p, i);
            }
        },
        &|p, error| {
            println!("{:?} , {:?}", p, error);
        },
    );
    match result {
        Ok(()) => println!("Search successfully"),
        Err(e) => println!("Failed Search, {}", e),
    }

    Ok(())
}

fn search_dir<P, Func, ErrFunc>(
    path: P,
    regex: &Regex,
    function: &Func,
    error_function: &ErrFunc,
) -> Result<(), Error>
where
    P: AsRef<Path>,
    Func: Fn(&Path, Vec<Match>),
    ErrFunc: Fn(&Path, Error),
{
    let path = path.as_ref();
    let meta_data = fs::metadata(path)?;
    if meta_data.is_dir() {
        // Read inside the directory
        let inner_dir = fs::read_dir(path)?;
        for inner in inner_dir.into_iter() {
            if let Err(e) = search_dir(inner?.path(), regex, function, error_function) {
                error_function(path, e);
            }
        }
    }

    if meta_data.is_file() {
        match search_file(path, regex) {
            Ok(data_matches) => function(path, data_matches),
            Err(e) => error_function(path, e),
        }
    }
    Ok(())
}

fn search_file<P>(path: P, r: &Regex) -> Result<Vec<Match>, Error>
where
    P: AsRef<Path> + Copy,
{
    let byte_data = fs::read(path)?;
    let data = std::str::from_utf8(&byte_data)?;
    let path = path.as_ref().clone();
    let mut match_data = Vec::new();

    for (i, line) in data.to_string().lines().enumerate() {
        if r.is_match(line) {
            let matches = Match {
                line: line.to_string(),
                line_num: i,
                file_name: path.to_owned(),
                regex: r.clone(),
            };
            match_data.push(matches);
        }
    }

    Ok(match_data)
}
