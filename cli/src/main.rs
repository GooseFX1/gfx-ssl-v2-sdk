use clap::Parser;
use gfx_ssl_v2_cli::Opt;

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    opt.process()?;
    Ok(())
}
