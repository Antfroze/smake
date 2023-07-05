use mlua::{prelude::*, Function, Table, Variadic};
use std::process::Command;

use crate::cli;

pub fn init(lua: &Lua) -> LuaResult<()> {
    register(lua)?;
    let matches = cli(&lua).get_matches();

    let table = lua.globals().get::<_, Table>("smake")?;

    let cmd = matches.subcommand();

    let (command, args) = cmd.unwrap();

    let method = table.get::<_, Function>(command)?;

    let cmd_args = args
        .get_raw("args")
        .into_iter()
        .flatten()
        .map(|str| str.to_string_lossy())
        .collect::<Vec<_>>();

    method
        .call::<_, Option<String>>(Variadic::from_iter(cmd_args))?
        .unwrap_or_default();

    Ok(())
}

pub fn register(lua: &Lua) -> LuaResult<()> {
    let platform = std::env::consts::OS;

    lua.globals().set("platform", platform)?;

    lua.globals().set("smake", lua.create_table()?)?;
    lua.globals().set("run", lua.create_function(run)?)?;
    lua.globals().set("runIn", lua.create_function(run_in)?)?;
    lua.globals().set("import", lua.create_function(import)?)?;

    let script = std::fs::read_to_string("./smake.lua").map_err(|e| {
        LuaError::RuntimeError(format!("Failed to read smake.lua: {}.", e).to_string())
    })?;

    lua.load(&script).exec()?;

    Ok(())
}

pub fn import(lua: &Lua, file_name: String) -> LuaResult<LuaValue> {
    lua.globals().set("Plugin", lua.create_table()?)?;
    let script = std::fs::read_to_string(format!("./plugins/{}.lua", file_name)).map_err(|e| {
        LuaError::RuntimeError(format!("Failed to load plugin: {}.", e).to_string())
    })?;
    lua.load(&script).exec()?;

    let import_fn: Function = lua
        .globals()
        .get::<_, Table>("Plugin")?
        .get("Import")
        .map_err(|e| {
            LuaError::RuntimeError(format!("Failed to load plugin: {}.", e).to_string())
        })?;
    let cloned_fn = import_fn.clone();
    let result = cloned_fn.call(())?;

    lua.globals().raw_remove("Plugin")?;

    Ok(result)
}

pub fn run(_: &Lua, cmds: Variadic<String>) -> LuaResult<()> {
    for cmd in cmds.iter() {
        Command::new("sh").arg("-c").arg(cmd).status()?;
    }

    Ok(())
}

pub fn run_in(_: &Lua, args: Variadic<String>) -> LuaResult<()> {
    let (path, cmds) = match args.as_slice() {
        [path, cmds @ ..] => (path, cmds),
        _ => {
            return Err(LuaError::RuntimeError(
                "Missing path or command(s) argument.".to_string(),
            ))
        }
    };

    for cmd in cmds.iter() {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(path)
            .status()?;
    }

    Ok(())
}
