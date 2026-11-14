//! `@pasta_calendar` module: holidays and the ghost's event table for Lua.
//!
//! ```lua
//! local cal = require "@pasta_calendar"
//! cal.holiday(2027, 1, 11) --> "成人の日"
//! cal.event(2027, 1, 1)    --> name from calendar.csv, or nil
//! ```

use mlua::{Lua, Result as LuaResult, Table, Value};
use pasta_core::{CalendarTable, holiday_name};

/// Take one integer argument; anything else (float, string, nil) is a Lua error.
fn int_arg<T: TryFrom<i64>>(fname: &str, pos: usize, v: &Value) -> LuaResult<T> {
    let n = match v {
        Value::Integer(n) => Some(*n),
        Value::Number(f) if f.fract() == 0.0 => Some(*f as i64),
        _ => None,
    };
    n.and_then(|n| T::try_from(n).ok()).ok_or_else(|| {
        mlua::Error::runtime(format!(
            "bad argument #{pos} to '{fname}' (integer expected, got {})",
            v.type_name()
        ))
    })
}

fn ymd(fname: &str, (y, m, d): (Value, Value, Value)) -> LuaResult<(i32, u32, u32)> {
    Ok((
        int_arg(fname, 1, &y)?,
        int_arg(fname, 2, &m)?,
        int_arg(fname, 3, &d)?,
    ))
}

/// Build the `{ holiday = fn, event = fn }` table. `event` captures `table`.
pub fn register(lua: &Lua, table: CalendarTable) -> LuaResult<Table> {
    let module = lua.create_table()?;
    module.set(
        "holiday",
        lua.create_function(|_, args| {
            let (y, m, d) = ymd("holiday", args)?;
            Ok(holiday_name(y, m, d))
        })?,
    )?;
    module.set(
        "event",
        lua.create_function(move |_, args| {
            let (y, m, d) = ymd("event", args)?;
            Ok(table.event_name(y, m, d).map(str::to_owned))
        })?,
    )?;
    Ok(module)
}
