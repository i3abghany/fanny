use rand::Rng;
use std::io::{self, Write};
use std::process::Command;
use std::process::Stdio;
use std::{fs, fs::File};

fn fuzz(tmpfname: &str) -> Result<bool, io::Error> {
    let status = Command::new("./objdump")
        .arg("-x")
        .arg(tmpfname)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    match status.code() {
        Some(_) => return Ok(false),
        None => return Ok(true),
    };
}

fn generate_input_file(corpse: &Vec<Vec<u8>>) -> Result<&str, io::Error> {
    let num = rand::thread_rng().gen_range(0..corpse.len());
    let sample_input = &corpse[num];
    let mut sample_input = sample_input.clone();

    for _ in 0..rand::thread_rng().gen_range(0..8) {
        let byte_to_mutate = rand::thread_rng().gen_range(0..sample_input.len());
        sample_input[byte_to_mutate] = rand::thread_rng().gen_range(0..255) as u8;
    }

    let tmpfname = "temp";
    let mut wf = File::create(tmpfname)?;
    wf.write(&sample_input)?;
    return Ok(tmpfname);
}

fn main() -> io::Result<()> {
    let mut corpse: Vec<Vec<u8>> = Vec::new();
    let corpus_path = "./corpus";
    let mut file_names = Vec::<String>::new();
    for file_entry in fs::read_dir(corpus_path)? {
        let file_entry = file_entry?;
        let path = file_entry.path();
        file_names.push(String::from(path.to_str().unwrap()));
        let read_data = match fs::read(path) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        corpse.push(read_data);
    }

    let mut n_runs = 0;
    let mut n_crashes = 0;
    const BATCH_SIZE: usize = 100;
    const BATCHES: usize = BATCH_SIZE * 20000;

    for _ in 0..BATCHES {
        n_runs = n_runs + 1;
        if n_runs % 100 == 0 {
            print!("Crashes: {}\n", n_crashes);
        }

        if fuzz(generate_input_file(&corpse)?)? {
            n_crashes += 1;
        }
    }

    Ok(())
}
