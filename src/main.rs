use console::{style, Term};
use rand::Rng;
use std::io::{self, Write};
use std::process::Command;
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::{fs, fs::File};

fn fuzz(tmpfname: &str) -> Result<bool, io::Error> {
    let status = Command::new("./objdump")
        .arg("-x")
        .arg(tmpfname)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    // "On Unix, this will return None if the process was terminated by a signal"
    Ok(status.code().is_none())
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
    const BATCHES: usize = 2_000_000_000;
    let start = Instant::now();

    let term = Term::stdout();
    for _ in 0..BATCHES {
        n_runs = n_runs + 1;
        if n_runs % BATCH_SIZE == 0 {
            let duration = start.elapsed();
            term.write_line(
                &format!(
                    "Crashes: {}, fcps: {}",
                    style(n_crashes).cyan(),
                    style(n_runs as f64 / (duration.as_millis() as f64 / 1000.0)).cyan()
                )[..],
            )?;
            term.move_cursor_up(1)?;
        }

        if fuzz(generate_input_file(&corpse)?)? {
            n_crashes += 1;
        }
    }

    Ok(())
}
