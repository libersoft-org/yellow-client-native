use rustyscript::{Runtime, RuntimeOptions, serde_json::Value};


#[tauri::command]
pub fn rusty_test() -> Result<(), String> {
    println!("Creating 3 separate JavaScript runtimes with isolated global state...\n");

    // Create 3 separate runtimes
    let mut runtime1 = Runtime::new(RuntimeOptions::default()).map_err(|e| e.to_string())?;
    let mut runtime2 = Runtime::new(RuntimeOptions::default()).map_err(|e| e.to_string())?;
    let mut runtime3 = Runtime::new(RuntimeOptions::default()).map_err(|e| e.to_string())?;

    // JavaScript code that sets a global variable and provides functions to manipulate it
    let js_code = r#"
        // Initialize global counter for this runtime
        globalThis.counter = 0;
        globalThis.runtimeName = '';

        function setRuntimeName(name) {
            globalThis.runtimeName = name;
            console.log(`Runtime ${name} initialized`);
        }

        function incrementCounter() {
            globalThis.counter++;
            console.log(`Runtime ${globalThis.runtimeName}: counter = ${globalThis.counter}`);
            return globalThis.counter;
        }

        function getState() {
            return {
                name: globalThis.runtimeName,
                counter: globalThis.counter
            };
        }

        function setCounter(value) {
            globalThis.counter = value;
            console.log(`Runtime ${globalThis.runtimeName}: counter set to ${value}`);
        }
    "#;

    // Load the same code into all 3 runtimes
    runtime1.eval::<()>(js_code).map_err(|e| e.to_string())?;
    runtime2.eval::<()>(js_code).map_err(|e| e.to_string())?;
    runtime3.eval::<()>(js_code).map_err(|e| e.to_string())?;

    // Set different runtime names for each
    let _: () = runtime1.call_function(None, "setRuntimeName", &[Value::from("Runtime-A")]).map_err(|e| e.to_string())?;
    let _: () = runtime2.call_function(None, "setRuntimeName", &[Value::from("Runtime-B")]).map_err(|e| e.to_string())?;
    let _: () = runtime3.call_function(None, "setRuntimeName", &[Value::from("Runtime-C")]).map_err(|e| e.to_string())?;

    println!("\n--- Demonstrating separate global state ---");

    // Set different counter values to further demonstrate isolation
    println!("\nSetting different counter values:");
    let _: () = runtime1.call_function(None, "setCounter", &[Value::from(100)]).map_err(|e| e.to_string())?;
    let _: () = runtime2.call_function(None, "setCounter", &[Value::from(200)]).map_err(|e| e.to_string())?;
    let _: () = runtime3.call_function(None, "setCounter", &[Value::from(300)]).map_err(|e| e.to_string())?;

    // Increment counters different amounts in each runtime
    println!("\nIncrementing Runtime-A 3 times:");
    for _ in 0..3 {
        let _: i32 = runtime1.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    println!("\nIncrementing Runtime-B 5 times:");
    for _ in 0..5 {
        let _: i32 = runtime2.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    println!("\nIncrementing Runtime-C 2 times:");
    for _ in 0..2 {
        let _: i32 = runtime3.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    // Get final state from each runtime
    println!("\n--- Final State of Each Runtime ---");

    let state1: Value = runtime1.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let state2: Value = runtime2.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let state3: Value = runtime3.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;

    println!("Runtime 1 state: {}", state1);
    println!("Runtime 2 state: {}", state2);
    println!("Runtime 3 state: {}", state3);





    // Increment counters different amounts in each runtime
    println!("\nIncrementing Runtime-A 3 times:");
    for _ in 0..3 {
        let _: i32 = runtime1.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    println!("\nIncrementing Runtime-B 5 times:");
    for _ in 0..5 {
        let _: i32 = runtime2.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    println!("\nIncrementing Runtime-C 2 times:");
    for _ in 0..2 {
        let _: i32 = runtime3.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    }

    // Get final state from each runtime
    println!("\n--- Final State of Each Runtime ---");

    let state1b: Value = runtime1.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let state2b: Value = runtime2.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let state3b: Value = runtime3.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;

    println!("Runtime 1 state: {}", state1b);
    println!("Runtime 2 state: {}", state2b);
    println!("Runtime 3 state: {}", state3b);




    // Demonstrate that modifying one doesn't affect others
    println!("\n--- Demonstrating Independence ---");
    println!("Incrementing only Runtime-A:");
    let _: i32 = runtime1.call_function(None, "incrementCounter", &Vec::<Value>::new()).map_err(|e| e.to_string())?;

    // Check that others remain unchanged
    let final_state1: Value = runtime1.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let final_state2: Value = runtime2.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;
    let final_state3: Value = runtime3.call_function(None, "getState", &Vec::<Value>::new()).map_err(|e| e.to_string())?;

    println!("After incrementing only Runtime-A:");
    println!("Runtime 1 final: {}", final_state1);
    println!("Runtime 2 final: {}", final_state2);
    println!("Runtime 3 final: {}", final_state3);

    println!("\nDemo complete! Each runtime maintained separate global state.");

    Ok(())
}
