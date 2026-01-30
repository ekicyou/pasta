#![allow(clippy::trivially_copy_pass_by_ref)]
use crate::error::MyResult;
use crate::util::parsers::req::{Parser, Rule};
use pasta_lua::mlua::{Lua, Table};
use pest::Parser as _;
use pest::iterators::FlatPairs;
use time::OffsetDateTime;

/// 指定時刻でdateテーブルを作成します。
/// テスト時に固定時刻を渡せるようにするための基盤関数です。
pub fn lua_date_from(lua: &Lua, dt: OffsetDateTime) -> MyResult<Table> {
    let t = lua.create_table()?;
    t.set("unix", dt.unix_timestamp())?;
    t.set("year", dt.year())?;
    t.set("month", dt.month() as u8)?;
    t.set("day", dt.day())?;
    t.set("hour", dt.hour())?;
    t.set("min", dt.minute())?;
    t.set("sec", dt.second())?;
    t.set("ns", dt.nanosecond())?;

    let ordinal = dt.ordinal();
    let num_days_from_sunday = dt.weekday().number_days_from_sunday();

    t.set("yday", ordinal)?;
    t.set("ordinal", ordinal)?;
    t.set("wday", num_days_from_sunday)?;
    t.set("num_days_from_sunday", num_days_from_sunday)?;
    Ok(t)
}

/// 現在時刻でdateテーブルを作成します。
#[allow(dead_code)]
pub fn lua_date(lua: &Lua) -> MyResult<Table> {
    let now = OffsetDateTime::now_local()?;
    lua_date_from(lua, now)
}

/// SHIORI REQUESTを解析し、luaオブジェクトに展開します。
/// * req.method: get / notify
/// * req.version: 30であること
/// * req.charset: utf-8であること
/// * req.id: event id
/// * req.base_id:
/// * req.status:
/// * req.security_level:
/// * req.sender:
/// * req.reference[num]: reference0～n
/// * req.dic[key]: 全ての値を辞書テーブルで保管
pub fn parse_request(lua: &Lua, text: &str) -> MyResult<Table> {
    let t = lua.create_table()?;
    t.set("reference", lua.create_table()?)?;
    t.set("dic", lua.create_table()?)?;
    t.set("date", lua_date(lua)?)?;
    let it = Parser::parse(Rule::req, text)?.flatten();
    parse1(&t, it)?;
    Ok(t)
}

fn parse1(table: &Table, mut it: FlatPairs<'_, Rule>) -> MyResult<()> {
    let pair = match it.next() {
        Some(a) => a,
        None => return Ok(()),
    };
    let rule = pair.as_rule();
    match rule {
        Rule::key_value => parse_key_value(table, &mut it)?,
        Rule::get => table.set("method", "get")?,
        Rule::notify => table.set("method", "notify")?,
        Rule::header3 => table.set("version", 30)?,
        Rule::shiori2_id => table.set("id", pair.as_str())?,
        Rule::shiori2_ver => {
            let version = {
                let nums: i32 = pair.as_str().parse().unwrap();
                if nums < 0 {
                    20
                } else if nums > 9 {
                    29
                } else {
                    nums + 20
                }
            };
            table.set("version", version)?
        }
        _ => (),
    };
    parse1(table, it)
}

fn parse_key_value(table: &Table, it: &mut FlatPairs<'_, Rule>) -> MyResult<()> {
    let pair = it.next().unwrap();
    let rule = pair.as_rule();
    let key = pair.as_str();
    let reference: Table = table.get("reference")?;
    let dic: Table = table.get("dic")?;

    let value = match rule {
        Rule::key_ref => {
            let nums: i32 = it.next().unwrap().as_str().parse().unwrap();
            let value = it.next().unwrap().as_str();
            reference.set(nums, value)?;
            value
        }
        _ => {
            let value = it.next().unwrap().as_str();
            match rule {
                Rule::key_charset => table.set("charset", value)?,
                Rule::key_id => table.set("id", value)?,
                Rule::key_base_id => table.set("base_id", value)?,
                Rule::key_status => table.set("status", value)?,
                Rule::key_security_level => table.set("security_level", value)?,
                Rule::key_sender => table.set("sender", value)?,
                Rule::key_other => (),
                _ => panic!(),
            };
            value
        }
    };
    dic.set(key, value)?;
    Ok(())
}
