//! Probe test: investigate why OnTalk never fires
//! This is a temporary diagnostic test.

mod common;

use common::copy_sample_ghost_to_temp;
use pasta::{PastaShiori, Shiori};

/// Probe 1: Check if the issue is time-based by examining Lua internal state.
/// We directly manipulate the virtual_dispatcher state via Lua to verify
/// the chain works when time is simulated.
#[test]
fn probe_ontalk_via_lua_state() {
    let temp = copy_sample_ghost_to_temp();
    let mut shiori = PastaShiori::default();

    // Load ghost
    assert!(shiori.load(1, temp.path().as_os_str()).unwrap());

    // Send OnBoot first
    let boot_req = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        Sender: SSP\r\n\
        SecurityLevel: local\r\n\
        ID: OnBoot\r\n\
        Reference0: master\r\n\
        \r\n";
    let boot_resp = shiori.request(boot_req).unwrap();
    println!("=== OnBoot Response ===");
    println!("{}", boot_resp);

    // Send first OnSecondChange to initialize dispatcher
    let req1 = "GET SHIORI/3.0\r\n\
        Charset: UTF-8\r\n\
        Sender: SSP\r\n\
        SecurityLevel: local\r\n\
        Status: balloon(0=0)\r\n\
        ID: OnSecondChange\r\n\
        Reference0: 1\r\n\
        \r\n";
    let resp1 = shiori.request(req1).unwrap();
    println!("=== First OnSecondChange Response ===");
    println!("{}", resp1);

    // Now examine the Lua state directly
    let runtime = shiori.runtime().expect("runtime should exist");
    let lua = runtime.lua();

    // Check virtual_dispatcher internal state
    let result = lua
        .load(
            r#"
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        local state = dispatcher._get_internal_state()
        
        local info = string.format(
            "next_hour_unix=%d, next_talk_time=%d, cached_config=%s",
            state.next_hour_unix,
            state.next_talk_time,
            tostring(state.cached_config ~= nil)
        )
        
        -- Get current time for comparison
        local os_time = os.time()
        info = info .. string.format(", os_time=%d", os_time)
        info = info .. string.format(", diff_to_talk=%d", state.next_talk_time - os_time)
        
        return info
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== Dispatcher State ===");
    println!("{}", result);

    // Check if SCENE.search can find OnTalk
    let scene_check = lua
        .load(
            r#"
        local SCENE = require("pasta.scene")
        local result = SCENE.search("OnTalk", nil, nil)
        if result then
            return string.format("FOUND: global=%s, local=%s, func_type=%s", 
                result.global_name, result.local_name, type(result.func))
        else
            return "NOT_FOUND"
        end
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== Scene Search Result ===");
    println!("{}", scene_check);

    // Check if co_exec works
    let co_exec_check = lua
        .load(
            r#"
        local SCENE = require("pasta.scene")
        local thread = SCENE.co_exec("OnTalk", nil, nil)
        if thread then
            return string.format("THREAD: type=%s, status=%s", type(thread), coroutine.status(thread))
        else
            return "NIL_THREAD"
        end
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== co_exec Result ===");
    println!("{}", co_exec_check);

    // Now manually force next_talk_time to the past and send another OnSecondChange
    lua.load(
        r#"
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        -- Force next_talk_time to 0 so it will be recalculated
        dispatcher._reset()
    "#,
    )
    .exec()
    .unwrap();

    // Send OnSecondChange to re-initialize
    let resp2 = shiori.request(req1).unwrap();
    println!("=== After Reset: OnSecondChange Response ===");
    println!("{}", resp2);

    // Now override next_talk_time to past
    let override_result = lua
        .load(
            r#"
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        local state = dispatcher._get_internal_state()
        return string.format("after re-init: next_talk_time=%d", state.next_talk_time)
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== After Re-init State ===");
    println!("{}", override_result);

    // The key issue: parse_request uses OffsetDateTime::now_local() for date.unix
    // So every request has the CURRENT time. If next_talk_time is 180+ seconds
    // in the future, it will never be reached without actually waiting.
    //
    // In SSP's real environment, 1 second passes between each OnSecondChange,
    // so after 180 seconds, the dispatcher should fire.
    //
    // Let's verify by directly calling check_talk with a future timestamp:
    let manual_fire = lua
        .load(
            r#"
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        local STORE = require("pasta.store")
        
        -- Get current state
        local state = dispatcher._get_internal_state()
        local future_time = state.next_talk_time + 1
        
        -- Create mock act with future time
        local ACT = require("pasta.shiori.act")
        local act = ACT.new(STORE.actors, {
            id = "OnSecondChange",
            status = "balloon(0=0)",
            date = { unix = future_time, hour = 12, min = 0, sec = 0 }
        })
        
        -- Call check_talk directly
        local result = dispatcher.check_talk(act)
        if result then
            return string.format("FIRED: type=%s", type(result))
        else
            return "NOT_FIRED"
        end
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== Manual check_talk with future time ===");
    println!("{}", manual_fire);

    // Now test via the full dispatch chain
    let full_dispatch = lua
        .load(
            r#"
        local dispatcher = require("pasta.shiori.event.virtual_dispatcher")
        local STORE = require("pasta.store")
        
        -- Reset to clean state
        dispatcher._reset()
        
        -- First call: initialize
        local ACT = require("pasta.shiori.act")
        local act1 = ACT.new(STORE.actors, {
            id = "OnSecondChange",
            status = "balloon(0=0)",
            date = { unix = 1000000, hour = 12, min = 0, sec = 0 }
        })
        dispatcher.dispatch(act1)
        
        -- Get state
        local state = dispatcher._get_internal_state()
        local info = string.format("After init: next_talk_time=%d, next_hour_unix=%d", 
            state.next_talk_time, state.next_hour_unix)
        
        -- Second call: with time past next_talk_time
        local act2 = ACT.new(STORE.actors, {
            id = "OnSecondChange",
            status = "balloon(0=0)",
            date = { unix = state.next_talk_time + 1, hour = 12, min = 5, sec = 0 }
        })
        local result = dispatcher.dispatch(act2)
        
        if result then
            info = info .. string.format("\nDISPATCH RESULT: type=%s", type(result))
            if type(result) == "thread" then
                -- Try to resume the thread
                local ok, value = coroutine.resume(result, act2)
                info = info .. string.format("\nRESUME: ok=%s, value_type=%s, value=%s", 
                    tostring(ok), type(value), tostring(value))
            end
        else
            info = info .. "\nDISPATCH RESULT: nil"
        end
        
        return info
    "#,
        )
        .eval::<String>()
        .unwrap();
    println!("=== Full Dispatch Chain Test ===");
    println!("{}", full_dispatch);

    // Assert basic findings
    assert!(
        scene_check.starts_with("FOUND"),
        "OnTalk scene should be findable: {}",
        scene_check
    );
}
