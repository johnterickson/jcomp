// extern crate strum;

use std::io::{self, BufRead};

use common::*;

fn main() -> Result<(), std::io::Error> {

    let lines = {
        let mut lines = Vec::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            lines.push(Line::parse(line));
        }
        lines
    };

    let rom = assemble(lines);

    simulate(&rom, 10000);

    Ok(())
}
