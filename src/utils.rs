use std::convert::AsRef;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::io::{BufReader, prelude::*};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;



extern crate chrono;
use chrono::Local;

use rand::{distr::Alphanumeric, RngExt};

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


pub fn to_file<S: AsRef<std::path::Path>>(filename: S, data: &Vec<u8>) -> io::Result<usize> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;
    file.set_len(data.len() as u64)?;
    file.write_all(&data[..])?;
    file.sync_all()?;
    Ok(data.len())
}



// there are corner cases with utf8 in filenames, so isolate all that here
fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    [path.as_os_str(), OsStr::new(suffix)]
        .into_iter()
        .collect::<OsString>()
        .into()
}


pub fn get_tempfile_with_prefix(prefix: &Path) -> io::Result<(File, PathBuf)> {
    const MAX_ATTEMPTS: usize = 1000;

    for attempt in 0..MAX_ATTEMPTS {
        let random_tag: String = rand::rng()
            .sample_iter(Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let proc_id = std::process::id();
        let temp = path_with_suffix(prefix, &format!(".{random_tag}-{attempt}-{proc_id}"));


        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => return Ok((file, temp)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(io::ErrorKind::AlreadyExists, "[could not create a temp file]"))
}

pub fn to_file_atomic(filename: impl AsRef<Path>, data: &[u8]) -> io::Result<usize> {
    fn write_then_move(
        mut file: File,
        data: &[u8],
        temp: &Path,
        destination: &Path,
    ) -> io::Result<usize> {
        file.write_all(data)?;
        file.sync_all()?;
        drop(file);
        fs::rename(temp, destination)?;
        Ok(data.len())
    }

    let destination = filename.as_ref();

    let prefix = path_with_suffix(destination, ".tmp");

    let (file, temp) = get_tempfile_with_prefix(&prefix)?;
    let result = write_then_move(file, data, &temp, destination);

    let status = if result.is_ok() { "ok" } else { "failed" };

    eprintln!(
        "[atomic_write_{status}: {result:?} | temp: {}, dest: {}]",
        temp.display(),
        destination.display(),
    );

    result
}





pub fn from_file<S: AsRef<std::path::Path>>(filename: S) -> io::Result<Vec<u8>> {
    let data = fs::read(filename)?;
    Ok(data)
}
