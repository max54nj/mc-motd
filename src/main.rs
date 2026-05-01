use std::env;

use color_eyre::Result;

use mc_motd::server::MOTDServer;

fn main() -> Result<()> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "full");
    }
    let server = MOTDServer::new(25565);
    server.start().unwrap();
    Ok(())
}
