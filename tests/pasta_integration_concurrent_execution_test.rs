//! Tests for concurrent execution of PastaEngine instances.
//!
//! These tests verify that PastaEngine instances can be safely moved across
//! thread boundaries and executed concurrently without data races.

mod common;

use common::{create_test_script, create_unique_persistence_dir, get_test_persistence_dir};
use pasta::{PastaEngine, ScriptEvent};
use std::thread;

#[test]
fn test_thread_safety() {
    // Test that engines can be created and executed in separate threads
    let script1 = r#"
＊test1
    さくら：スレッド1
"#;
    let script2 = r#"
＊test2
    うにゅう：スレッド2
"#;

    // Spawn threads that create and execute engines
    let handle1 = thread::spawn(move || {
        let script_dir = create_test_script(script1).expect("Failed to create script");
        let persistence_dir =
            create_unique_persistence_dir().expect("Failed to create persistence dir");
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
            .expect("Failed to create engine in thread 1");
        let events = engine
            .execute_label("test1")
            .expect("Failed to execute in thread 1");

        // Verify the result
        let has_sakura = events
            .iter()
            .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら"));
        assert!(has_sakura, "Thread 1 should have さくら speaker");
        events
    });

    let handle2 = thread::spawn(move || {
        let script_dir = create_test_script(script2).expect("Failed to create script");
        let persistence_dir =
            create_unique_persistence_dir().expect("Failed to create persistence dir");
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
            .expect("Failed to create engine in thread 2");
        let events = engine
            .execute_label("test2")
            .expect("Failed to execute in thread 2");

        // Verify the result
        let has_unyuu = events
            .iter()
            .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "うにゅう"));
        assert!(has_unyuu, "Thread 2 should have うにゅう speaker");
        events
    });

    // Wait for both threads to complete
    let events1 = handle1.join().expect("Thread 1 panicked");
    let events2 = handle2.join().expect("Thread 2 panicked");

    // Both should have produced events
    assert!(!events1.is_empty());
    assert!(!events2.is_empty());
}

#[test]
fn test_multiple_threads_same_script() {
    // Test that multiple threads can create engines from the same script
    let script = r#"
＊greeting
    さくら：こんにちは
"#;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let script_copy = script.to_string();
            thread::spawn(move || {
                let script_dir = create_test_script(&script_copy).expect("Failed to create script");
                let _persistence_dir = get_test_persistence_dir();
                let persistence_dir =
                    create_unique_persistence_dir().expect("Failed to create persistence dir");
                let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
                    .unwrap_or_else(|_| panic!("Failed to create engine in thread {}", i));
                let events = engine
                    .execute_label("greeting")
                    .unwrap_or_else(|_| panic!("Failed to execute in thread {}", i));

                assert!(!events.is_empty(), "Thread {} should produce events", i);
                events.len()
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        let event_count = handle.join().expect("Thread panicked");
        assert!(event_count > 0);
    }
}

#[test]
fn test_send_trait() {
    // Test that PastaEngine implements Send by moving it across thread boundary
    let script = r#"
＊test
    さくら：Send テスト
"#;

    let _script_dir = create_test_script(script).expect("Failed to create script");
    let _persistence_dir =
        create_unique_persistence_dir().expect("Failed to create persistence dir");
    let script_dir = create_test_script(script).expect("Failed to create script");
    let persistence_dir =
        create_unique_persistence_dir().expect("Failed to create persistence dir");
    let engine = PastaEngine::new(&script_dir, &persistence_dir).expect("Failed to create engine");

    // Move engine to another thread
    let handle = thread::spawn(move || {
        // This proves that PastaEngine implements Send
        // scene existence is verified by successful execution
        drop(engine);
    });

    handle.join().expect("Thread panicked");
}

#[test]
fn test_independent_execution_across_threads() {
    // Test that engines in different threads produce independent results
    let script = r#"
＊label_a
    さくら：ラベルA

＊label_b
    うにゅう：ラベルB
"#;

    let script1 = script.to_string();
    let script2 = script.to_string();

    let handle1 = thread::spawn(move || {
        let script_dir = create_test_script(&script1).expect("Failed to create script");
        let persistence_dir =
            create_unique_persistence_dir().expect("Failed to create persistence dir");
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
            .expect("Failed to create engine in thread 1");
        engine
            .execute_label("label_a")
            .expect("Failed to execute label_a")
    });

    let handle2 = thread::spawn(move || {
        let script_dir = create_test_script(&script2).expect("Failed to create script");
        let persistence_dir =
            create_unique_persistence_dir().expect("Failed to create persistence dir");
        let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
            .expect("Failed to create engine in thread 2");
        engine
            .execute_label("label_b")
            .expect("Failed to execute label_b")
    });

    let events1 = handle1.join().expect("Thread 1 panicked");
    let events2 = handle2.join().expect("Thread 2 panicked");

    // Verify independent execution - different speakers
    let has_sakura = events1
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "さくら"));
    let has_unyuu = events2
        .iter()
        .any(|e| matches!(e, ScriptEvent::ChangeSpeaker { name } if name == "うにゅう"));

    assert!(has_sakura, "Thread 1 should execute label_a");
    assert!(has_unyuu, "Thread 2 should execute label_b");
}

#[test]
fn test_concurrent_engine_creation() {
    // Test that many engines can be created concurrently
    let script = r#"
＊test
    さくら：並行生成テスト
"#;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let script_copy = script.to_string();
            thread::spawn(move || {
                let script_dir = create_test_script(&script_copy).expect("Failed to create script");
                let persistence_dir =
                    create_unique_persistence_dir().expect("Failed to create persistence dir");
                PastaEngine::new(&script_dir, &persistence_dir)
                    .unwrap_or_else(|_| panic!("Failed to create engine in thread {}", i))
            })
        })
        .collect();

    // Collect all engines
    let engines: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();

    // All should be valid
    assert_eq!(engines.len(), 10);
}

#[test]
fn test_no_data_races() {
    // Test structural safety: no global state means no data races possible
    // This test creates engines in multiple threads and verifies they all work correctly
    let script = r#"
＊counter
    さくら：カウント
"#;

    let handles: Vec<_> = (0..20)
        .map(|_| {
            let script_copy = script.to_string();
            thread::spawn(move || {
                let script_dir = create_test_script(&script_copy).expect("Failed to create script");
                let persistence_dir =
                    create_unique_persistence_dir().expect("Failed to create persistence dir");
                let mut engine = PastaEngine::new(&script_dir, &persistence_dir)
                    .expect("Failed to create engine");
                // Execute multiple times
                for _ in 0..3 {
                    let events = engine.execute_label("counter").expect("Failed to execute");
                    assert!(!events.is_empty());
                }
            })
        })
        .collect();

    // All threads should complete successfully
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_thread_local_cache() {
    // Test that each thread's engine has its own cache
    let script = r#"
＊cached
    さくら：キャッシュテスト
"#;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let script_copy = script.to_string();
            thread::spawn(move || {
                // Create two engines in the same thread
                let script_dir1 =
                    create_test_script(&script_copy).expect("Failed to create script");
                let persistence_dir1 =
                    create_unique_persistence_dir().expect("Failed to create persistence dir");
                let mut engine1 = PastaEngine::new(&script_dir1, &persistence_dir1)
                    .unwrap_or_else(|_| panic!("Failed to create engine1 in thread {}", i));
                let script_dir2 =
                    create_test_script(&script_copy).expect("Failed to create script");
                let persistence_dir2 =
                    create_unique_persistence_dir().expect("Failed to create persistence dir");
                let mut engine2 = PastaEngine::new(&script_dir2, &persistence_dir2)
                    .unwrap_or_else(|_| panic!("Failed to create engine2 in thread {}", i));

                let events1 = engine1.execute_label("cached").expect("Failed on engine1");
                let events2 = engine2.execute_label("cached").expect("Failed on engine2");

                assert!(!events1.is_empty());
                assert!(!events2.is_empty());
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
