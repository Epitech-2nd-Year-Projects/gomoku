use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut lines = stdin.lock().lines();

    while let Some(line) = lines.next() {
        match line {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                // TODO: Process the input
                if let Err(e) = writeln!(stdout, "RECEIVED: {}", input) {
                    eprintln!("Failed to write to stdout: {}", e);
                    break;
                }

                if let Err(e) = stdout.flush() {
                    eprintln!("Failed to flush stdout: {}", e);
                    break;
                }

                if input == "END" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}
