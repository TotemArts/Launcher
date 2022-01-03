use powershell_script;
use clap::Parser;

/// Used for setting folder permissions
#[derive(Parser, Debug)]
#[clap(about)]
struct Args {
    /// Directory of which to set the permissions
    #[clap(short, long)]
    directory: String,
}

fn main() {
    let args = Args::parse();

    match powershell_script::run(&format!("($acl = Get-ACL {directory}).AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule([System.Security.Principal.WindowsIdentity]::GetCurrent().Name,\"FullControl\",\"Allow\"))); $acl | Set-ACL {directory}", directory=args.directory), true) {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            println!("Could not set Access Rule for RenegadeX directory: {}", e);
        }
    }}
