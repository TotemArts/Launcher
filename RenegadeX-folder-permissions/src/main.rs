use powershell_script;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let script = &args[1];

    match powershell_script::run(script, true) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }}
