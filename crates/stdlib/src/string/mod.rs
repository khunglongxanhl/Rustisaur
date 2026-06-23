//! String utilities module.

mod functions;

use mlua::{Lua, Table};

pub fn register(lua: &Lua, rex: &Table) -> mlua::Result<()> {
    let string_mod = lua.create_table()?;

    // Basic functions
    string_mod.set(
        "upper",
        lua.create_function(|_, s: String| Ok(functions::upper(&s)))?,
    )?;

    string_mod.set(
        "lower",
        lua.create_function(|_, s: String| Ok(functions::lower(&s)))?,
    )?;

    string_mod.set(
        "trim",
        lua.create_function(|_, s: String| Ok(functions::trim(&s)))?,
    )?;

    string_mod.set(
        "trim_left",
        lua.create_function(|_, s: String| Ok(functions::trim_left(&s)))?,
    )?;

    string_mod.set(
        "trim_right",
        lua.create_function(|_, s: String| Ok(functions::trim_right(&s)))?,
    )?;

    string_mod.set(
        "split",
        lua.create_function(|lua, (s, delim): (String, String)| {
            let parts = functions::split(&s, &delim);
            let table = lua.create_table()?;
            for (i, part) in parts.into_iter().enumerate() {
                table.set(i + 1, part)?;
            }
            Ok(table)
        })?,
    )?;

    string_mod.set(
        "join",
        lua.create_function(|_, (table, delim): (mlua::Table, String)| {
            let parts: Vec<String> = table.sequence_values().filter_map(|v| v.ok()).collect();
            Ok(functions::join(&parts, &delim))
        })?,
    )?;

    string_mod.set(
        "replace",
        lua.create_function(|_, (s, from, to): (String, String, String)| {
            Ok(functions::replace(&s, &from, &to))
        })?,
    )?;

    string_mod.set(
        "replace_all",
        lua.create_function(|_, (s, from, to): (String, String, String)| {
            Ok(functions::replace_all(&s, &from, &to))
        })?,
    )?;

    string_mod.set(
        "starts_with",
        lua.create_function(|_, (s, prefix): (String, String)| {
            Ok(functions::starts_with(&s, &prefix))
        })?,
    )?;

    string_mod.set(
        "ends_with",
        lua.create_function(|_, (s, suffix): (String, String)| {
            Ok(functions::ends_with(&s, &suffix))
        })?,
    )?;

    string_mod.set(
        "contains",
        lua.create_function(|_, (s, pattern): (String, String)| {
            Ok(functions::contains(&s, &pattern))
        })?,
    )?;

    string_mod.set(
        "capitalize",
        lua.create_function(|_, s: String| Ok(functions::capitalize(&s)))?,
    )?;

    string_mod.set(
        "repeat",
        lua.create_function(|_, (s, count): (String, usize)| Ok(functions::repeat(&s, count)))?,
    )?;

    string_mod.set(
        "slice",
        lua.create_function(|_, (s, start, end): (String, usize, usize)| {
            Ok(functions::slice(&s, start, end))
        })?,
    )?;

    string_mod.set(
        "reverse",
        lua.create_function(|_, s: String| Ok(functions::reverse(&s)))?,
    )?;

    string_mod.set(
        "pad_left",
        lua.create_function(|_, (s, width, ch): (String, usize, String)| {
            let ch = ch.chars().next().unwrap_or(' ');
            Ok(functions::pad_left(&s, width, ch))
        })?,
    )?;

    string_mod.set(
        "pad_right",
        lua.create_function(|_, (s, width, ch): (String, usize, String)| {
            let ch = ch.chars().next().unwrap_or(' ');
            Ok(functions::pad_right(&s, width, ch))
        })?,
    )?;

    string_mod.set(
        "len",
        lua.create_function(|_, s: String| Ok(functions::len(&s)))?,
    )?;

    string_mod.set(
        "is_empty",
        lua.create_function(|_, s: String| Ok(functions::is_empty(&s)))?,
    )?;

    rex.set("string", string_mod)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper() {
        assert_eq!(functions::upper("hello"), "HELLO");
    }

    #[test]
    fn test_lower() {
        assert_eq!(functions::lower("WORLD"), "world");
    }

    #[test]
    fn test_trim() {
        assert_eq!(functions::trim("  hello  "), "hello");
    }

    #[test]
    fn test_split() {
        let parts = functions::split("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(functions::capitalize("hello"), "Hello");
    }

    #[test]
    fn test_reverse() {
        assert_eq!(functions::reverse("hello"), "olleh");
    }
}
