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
    /// Regex for search
    #[clap(value_parser)]
    regex: String,

    /// Target path
    #[clap(value_parser, short, long)]
    path: String,
}
#[derive(Debug, PartialEq, Eq)]

struct Match {
    line_num: usize,
    line: String,
    regex: String,
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
                regex: r.to_string(),
            };
            match_data.push(matches);
        }
    }

    Ok(match_data)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn find_in_exist_file() -> Result<(), Error> {
        let path = Path::new("test_data/test.txt");
        let regex = Regex::new("Hello")?;
        let matches = search_file(path, &regex)?;
        let correct_ans = vec![
            Match {
                line_num: 4,
                line: "I wave to others to say Hello".to_string(),
                regex: regex.to_string(),
                file_name: path.to_path_buf(),
            },
            Match {
                line_num: 6,
                line: "They say Hello back".to_string(),
                regex: regex.to_string(),
                file_name: path.to_path_buf(),
            },
        ];
        assert_eq!(correct_ans.iter().eq(matches.iter()), true);
        Ok(())
    }

    #[test]
    fn find_in_exist_dir() -> Result<(), Error> {
        let path = Path::new("test_data/");
        let regex = Regex::new("Hello")?;
        let matches = search_dir(path, &regex, &|_e, _p| {}, &|_e, _p| {})?;
        assert_eq!(matches, ());
        Ok(())
    }

    #[test]
    fn find_non_exist_file() -> Result<(), Error> {
        let path = Path::new("test_data/non_existed.txt");
        let regex = Regex::new("Hello")?;
        if let Err(e) = search_file(path, &regex) {
            assert_eq!(
                e.to_string(),
                "No such file or directory (os error 2)".to_string()
            );
        }
        Ok(())
    }

    #[test]
    fn find_non_exist_dir() -> Result<(), Error> {
        let path = Path::new("non_existed/");
        let regex = Regex::new("Hello")?;
        if let Err(e) = search_dir(path, &regex, &|_e, _p| {}, &|_e, _p| {}) {
            assert_eq!(
                e.to_string(),
                "No such file or directory (os error 2)".to_string()
            );
        }
        Ok(())
    }
}
