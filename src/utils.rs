use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::io::{BufReader, prelude::*};
use std::path::Path;
use std::process::Command;

use std::process::ExitStatus;

extern crate chrono;
use chrono::Local;

//type StrRef = dyn AsRef<OsStr>;

pub fn timetag() -> String {
    let now = Local::now();
    format!("{}", now.format("%Y-%m-%d.%H-%M-%S"))
}

pub fn shell<S: AsRef<OsStr>>(exe: &str, args: &[S]) -> Result<ExitStatus, std::io::Error> {
    let mut cmd = &mut Command::new(exe);
    for e in args {
        cmd = cmd.arg(&e)
    }
    cmd.status()
}

pub fn filestream(fname: &str, f: fn(&str) -> bool) -> u32 {
    let mut n: u32 = 0;
    let file = File::open(fname).ok().unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(e) = line {
            n += 1;
            if !f(&e) {
                return n;
            }
        }
    }
    n
}

pub fn instream(f: fn(&str) -> bool) -> u32 {
    let stdin = io::stdin();
    let mut n: u32 = 0;
    for line in stdin.lock().lines() {
        if let Ok(e) = line {
            n += 1;
            if !f(&e) {
                return n;
            }
        }
    }
    n
}

pub fn backfile(f: &str, tag: &str) -> std::io::Result<String> {
    let tagged = if tag == "" {
        format!("{0}.{1}", f, timetag())
    } else {
        format!("{0}.{1}", f, tag)
    };
    fs::copy(f, &tagged)?; // cp f tagged
    Ok(tagged)
}

pub fn stdin_line(line: &mut String) -> Result<usize, std::io::Error> {
    io::stdin().read_line(line)
}

pub fn fwrite_addbytes<T: AsRef<Path>>(f: &T, data: &Vec<u8>) {
    let mut file = OpenOptions::new().write(true).append(true).open(f).unwrap();
    if let Err(_e) = file.write_all(data) {
        eprintln!("__cannot_add_to_file__:{:?}", file.metadata());
        return;
    }
}

pub fn to_file<S: AsRef<std::path::Path>>(filename: S, data: &Vec<u8>) -> io::Result<usize> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;
    file.set_len(data.len() as u64)?;
    file.write_all(&data[..])?;
    Ok(data.len())
}

pub fn from_file<S: AsRef<std::path::Path>>(filename: S) -> io::Result<Vec<u8>> {
    let data = fs::read(filename)?;
    Ok(data)
}
