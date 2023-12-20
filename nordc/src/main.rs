mod eval;
mod parser;

use anyhow::Result;

fn main() -> Result<()> {
    // Simple eval loop
    loop {
        // Read
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // Eval
        let output = eval::eval_from_str(&input.trim());

        // Print
        match output {
            Ok(output) => println!("{:#?}", output),
            Err(e) => println!("Error: {}", e),
        }
    }
}
