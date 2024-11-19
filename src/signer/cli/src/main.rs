use clap::Parser;
use signer_cli::args::SignerCliArgs;
use signer_cli::execute;
use tokio::runtime::Builder;


fn main() {
    let args = SignerCliArgs::parse();
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Unable to create a runtime");
    runtime.block_on(async {
        if let Err(err) = execute(&args).await {
            println!("Failed to execute command: {}", err);
            std::process::exit(1);
        }
    });
}
