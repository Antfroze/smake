use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, Command};
use mlua::{Lua, Table, Value};

pub fn cli(lua: &Lua) -> Command {
    let mut cmd = Command::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!())
        .version(crate_version!())
        .subcommand_required(true);

    let table = lua.globals().get::<_, Table>("smake").unwrap();
    for pair in table.pairs::<String, Value>() {
        let (name, value) = pair.unwrap();
        if let Value::Function(_) = value {
            let static_name: &'static str = Box::leak(name.into_boxed_str());

            cmd = cmd.subcommand(
                Command::new(static_name).arg(
                    Arg::new("args")
                        .num_args(0..)
                        .required(false)
                        .help(format!("Arguments for the {} command.", static_name)),
                ),
            );
        }
    }

    cmd
}
