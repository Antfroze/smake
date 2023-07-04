extern crate shell_words;

mod api;
mod cli;

use cli::cli;
use mlua::prelude::*;

fn main() -> LuaResult<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;

    api::init(&lua)?;

    Ok(())
}
