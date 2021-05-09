use anyhow::Result;

mod codegen;
mod flags;

fn main() -> Result<()> {
    let flags = flags::Xtask::from_env()?;

    match flags.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
            Ok(())
        }
        flags::XtaskCmd::Codegen(codegen) => codegen.run(),
    }
}
