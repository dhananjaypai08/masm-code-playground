use miden_processor::ExecutionOptions;
use miden_stdlib::StdLibrary;
use miden_vm::{
    assembly::DefaultSourceManager, execute, prove, AdviceInputs, Assembler, DefaultHost,
    ProvingOptions, StackInputs,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

#[derive(Deserialize)]
pub struct ExecutionRequest {
    pub program: String,
    pub inputs: Option<Value>,
}

#[derive(Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stack_outputs: Option<Vec<String>>,
    pub program_hash: Option<String>,
    pub cycles: Option<u32>,
    pub error: Option<String>,
    pub compilation_time_ms: Option<f64>,
    pub execution_time_ms: Option<f64>,
    pub total_time_ms: Option<f64>,
}

#[derive(Serialize)]
pub struct ProofResult {
    pub success: bool,
    pub proof_bytes: Option<Vec<u8>>,
    pub program_hash: Option<String>,
    pub stack_outputs: Option<Vec<String>>,
    pub error: Option<String>,
    pub compilation_time_ms: Option<f64>,
    pub proving_time_ms: Option<f64>,
    pub total_time_ms: Option<f64>,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! Welcome to Miden VM Playground!")
}

#[tauri::command]
pub fn instantiate() -> String {
    let source_manager = Arc::new(DefaultSourceManager::default());

    let assembler = Assembler::default()
        .with_debug_mode(true)
        .with_static_library(StdLibrary::default())
        .unwrap();

    let program = assembler
        .assemble_program("begin push.8 push.5 add swap drop end")
        .unwrap();

    let stack_inputs = StackInputs::default();
    let advice_inputs = AdviceInputs::default();
    let mut host = DefaultHost::default();
    let exec_options = ExecutionOptions::default();

    let trace = execute(
        &program,
        stack_inputs.clone(),
        advice_inputs.clone(),
        &mut host,
        exec_options,
        source_manager.clone(),
    )
    .unwrap();

    let stack_top: Vec<_> = trace
        .stack_outputs()
        .stack_truncated(1)
        .iter()
        .map(|f| f.to_string())
        .collect();
    stack_top[0].to_string()
}

#[tauri::command]
pub fn exec_program(program: &str) -> Result<String, String> {
    exec_program_with_inputs(program, None)
}

#[tauri::command]
pub fn exec_program_with_inputs(
    program: &str,
    inputs_json: Option<String>,
) -> Result<String, String> {
    let result = execute_program_impl(program, inputs_json.as_deref());
    Ok(serde_json::to_string(&result).unwrap())
}

#[tauri::command]
pub fn generate_proof_with_inputs(
    program: &str,
    inputs_json: Option<String>,
) -> Result<String, String> {
    let result = generate_proof_impl(program, inputs_json.as_deref());
    Ok(serde_json::to_string(&result).unwrap())
}

#[tauri::command]
pub fn get_example_programs() -> String {
    let prime_generator = r#"use.std::sys

# append the current number to the prime list
proc.append
    # initial state
    # [prime, i, n, primes..]

    # [prime, prime, i, n, primes..]
    dup

    # [i, prime, prime, i, n, primes..]
    dup.2

    # [prime, i, n, primes..]
    mem_store

    # [i++, n, primes..]
    swap.2
    swap
    add.1
end

# push a boolean on whether or not the program should continue
proc.should_continue
    # initial state
    # [i, n, primes..]

    # [i, n, i, n, primes..]
    dup.1
    dup.1

    # [should_continue, i, n, primes..]
    neq
end

# define if check should continue
# will return two flags: one if the loop should continue, the other if candidate is prime
proc.is_not_prime_should_continue
    # initial state
    # [j, candidate, i, n, primes..]

    # load the current prime
    # [prime, j, candidate, i, n, primes..]
    dup
    mem_load

    # push return flags
    # [continue loop?, is prime?, prime, j, candidate, i, n, primes..]
    push.0.1

    # a composite number have its smallest prime squared lesser than itself.
    # if the squared prime is bigger than the candidate, and provided we iterate
    # a list of ordered primes, then the number is a prime.
    #
    # this will also protect the algorithm from overflowing the list of current list of primes
    # because the squared prime will always halt the iteration before the end of the list is
    # reached
    #
    # [squared prime, continue loop?, is prime?, prime, j, candidate, i, n, primes..]
    dup.2
    dup
    mul
    # [candidate, squared prime, continue loop?, is prime?, prime, j, candidate, i, n, primes..]
    dup.5
    # [continue loop?, is prime?, prime, j, candidate, i, n, primes..]
    gt
    if.true
        drop
        drop
        push.1.0
    end

    # check mod only if should continue loop
    dup
    if.true
        # [remainder, continue loop?, is prime?, prime, j, candidate, i, n, primes..]
        dup.4
        dup.3
        u32assert2 u32mod

        # if remainder is zero, then the number is divisible by prime; hence isn't prime
        # [continue loop?, is prime?, prime, j, candidate, i, n, primes..]
        eq.0
        if.true
            drop
            drop
            push.0.0
        end
    end

    # [continue loop?, is prime?, j, candidate, i, n, primes..]
    swap.2
    drop
    swap
end

# check if current candidate isn't a prime
proc.is_not_prime
    # initial state
    # [candidate, i, n, primes..]

    # create a counter `j` to iterate over primes
    # [j, candidate, i, n, primes..]
    push.0

    exec.is_not_prime_should_continue
    while.true
        # [j, candidate, i, n, primes..]
        drop
        add.1

        # [is prime?, j, candidate, i, n, primes..]
        exec.is_not_prime_should_continue
    end

    # [is not prime?, candidate, i, n, primes..]
    swap
    drop
    eq.0
end

# calculate and push next prime to the stack
proc.next
    # initial state
    # [i, n, primes..]

    # create a candidate
    # [candidate, i, n, primes..]
    dup.2
    add.2

    exec.is_not_prime
    while.true
        # [candidate, i, n, primes..]
        add.2
        exec.is_not_prime
    end

    # [i, n, primes..]
    exec.append
end

# the stack is expected to contain on its top the desired primes count. this can be achieved via the
# *.inputs file.
#
# the end of the program will return a stack containing all the primes, up to the nth argument.
#
# example:
#
# input:
# [50, ..]
#
# output:
# [229, 227, 223, 211, 199, 197, 193, 191, 181, 179, 173, 167, 163, 157, 151, 149]
begin
    # create a counter `i`
    push.0

    # 2 and 3 are the unique sequential primes. by pushing these manually, we can iterate
    # the candidates in chunks of 2

    # append first known prime
    push.2
    exec.append

    # append second known prime
    push.3
    exec.append

    # find next primes until limit is reached
    exec.should_continue
    while.true
        exec.next
        exec.should_continue
    end

    # drop the counters
    drop
    drop

    # Truncate stack to make constraints happy
    exec.sys::truncate_stack
end"#;

    let examples = vec![
        ("Basic Addition", "# Simple addition example\n# Pushes 3 and 5, then adds them\n# Result: 8 on stack top, 3 below it\nbegin\n    push.3\n    push.5\n    add\n    swap\n    drop\nend"),
        ("Input Stack Demo", "# Demonstrates using input stack values\n# Takes two numbers from input and adds them\n# Try with: [\"10\", \"20\"] → Result: 30\nbegin\n    # Input values are already on stack\n    # Stack: [20, 10] (top to bottom)\n    add\n    # Stack: [30]\nend"),
        ("Fibonacci Numbers", "# Generates first 8 Fibonacci numbers\n# Uses iterative approach with stack manipulation\n# Result: F(8)=21 on top, previous numbers below\nbegin\n    push.1 push.1  # Start with F(0)=1, F(1)=1\n    push.6         # Generate 6 more numbers\n    repeat.6\n        dup.1      # Duplicate second element\n        add        # Add top two elements\n        swap       # Swap for next iteration\n    end\n    drop           # Remove counter\nend"),
        ("Prime Generator", prime_generator),
        ("Conditional Logic", "# Compares two numbers and keeps the larger one\n# Demonstrates if-else branching\n# Compares 15 and 10, keeps 15\nbegin\n    push.15 push.10  # Stack: [10, 15]\n    dup.1 gt         # Check if 15 > 10\n    if.true\n        swap         # Put larger number on top\n    end\n    drop             # Remove smaller number\nend"),
        ("Memory Operations", "# Shows memory store and load operations\n# Stores values in memory addresses 0 and 1\n# Then loads and adds them\nbegin\n    # Store 42 at address 0\n    push.42 push.0 mem_store\n    # Store 100 at address 1  \n    push.100 push.1 mem_store\n    # Load both values and add\n    push.0 mem_load    # Load from address 0\n    push.1 mem_load    # Load from address 1\n    add                # 42 + 100 = 142\nend"),
        ("Stack Manipulation", "# Demonstrates various stack operations\n# Shows dup, swap, drop operations\n# Final result: [3, 3] (duplicate 3s)\nbegin\n    push.1 push.2 push.3 push.4  # Stack: [4,3,2,1]\n    swap.2                       # Swap top with 3rd: [2,3,4,1]\n    drop                         # Remove top: [3,4,1]\n    dup                          # Duplicate top: [3,3,4,1]\n    swap.2                       # Final: [3,3,1,4]\n    drop drop                    # Keep only: [3,3]\nend"),
        ("Counter with Input", "# Takes input number and adds 5 to it\n# Simple arithmetic with input\n# Try with: [\"7\"] → Result: 12\nbegin\n    # Input value is on stack\n    push.5       # Add 5 to it\n    add          # Perform addition\nend"),
    ];

    serde_json::to_string(&examples).unwrap()
}

fn parse_stack_inputs(inputs_json: &str) -> Result<StackInputs, String> {
    let parsed: Value =
        serde_json::from_str(inputs_json).map_err(|e| format!("Invalid JSON: {e}"))?;

    let mut inputs = Vec::new();

    if let Some(operand_stack) = parsed.get("operand_stack") {
        if let Some(stack_array) = operand_stack.as_array() {
            for item in stack_array {
                if let Some(val_str) = item.as_str() {
                    let val: u64 = val_str
                        .parse()
                        .map_err(|e| format!("Invalid number '{val_str}': {e}"))?;
                    inputs.push(val);
                } else if let Some(val_num) = item.as_u64() {
                    inputs.push(val_num);
                }
            }
        }
    }

    // Reverse because stack inputs are pushed in reverse order
    inputs.reverse();

    StackInputs::try_from_ints(inputs).map_err(|e| format!("Failed to create stack inputs: {e}"))
}

pub fn execute_program_impl(program: &str, inputs_json: Option<&str>) -> ExecutionResult {
    let total_start = Instant::now();
    let source_manager = Arc::new(DefaultSourceManager::default());

    // Create assembler with source manager and standard library
    let compilation_start = Instant::now();
    let assembler = match Assembler::default()
        .with_debug_mode(true)
        .with_static_library(StdLibrary::default())
    {
        Ok(asm) => asm,
        Err(e) => {
            return ExecutionResult {
                success: false,
                stack_outputs: None,
                program_hash: None,
                cycles: None,
                error: Some(format!("Failed to configure assembler: {e}")),
                compilation_time_ms: None,
                execution_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    // Parse and set up stack inputs
    let stack_inputs = if let Some(inputs_str) = inputs_json {
        match parse_stack_inputs(inputs_str) {
            Ok(inputs) => inputs,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    stack_outputs: None,
                    program_hash: None,
                    cycles: None,
                    error: Some(e),
                    compilation_time_ms: None,
                    execution_time_ms: None,
                    total_time_ms: Some(total_start.elapsed().as_millis() as f64),
                }
            }
        }
    } else {
        StackInputs::default()
    };

    // Assemble the program
    let program = match assembler.assemble_program(program) {
        Ok(prog) => prog,
        Err(e) => {
            return ExecutionResult {
                success: false,
                stack_outputs: None,
                program_hash: None,
                cycles: None,
                error: Some(format!("Assembly error: {e}")),
                compilation_time_ms: Some(compilation_start.elapsed().as_millis() as f64),
                execution_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    let compilation_time = compilation_start.elapsed().as_millis() as f64;

    let advice_inputs = AdviceInputs::default();
    let mut host = DefaultHost::default();
    let exec_options = ExecutionOptions::default();

    // Execute the program
    let execution_start = Instant::now();
    let trace = match execute(
        &program,
        stack_inputs.clone(),
        advice_inputs.clone(),
        &mut host,
        exec_options,
        source_manager.clone(),
    ) {
        Ok(trace) => trace,
        Err(e) => {
            return ExecutionResult {
                success: false,
                stack_outputs: None,
                program_hash: None,
                cycles: None,
                error: Some(format!("Execution error: {e}")),
                compilation_time_ms: Some(compilation_time),
                execution_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    let execution_time = execution_start.elapsed().as_millis() as f64;
    let total_time = total_start.elapsed().as_millis() as f64;

    // Get stack outputs (show more elements)
    let stack_outputs: Vec<String> = trace
        .stack_outputs()
        .stack_truncated(16)
        .iter()
        .map(|f| f.to_string())
        .collect();

    ExecutionResult {
        success: true,
        stack_outputs: Some(stack_outputs),
        program_hash: Some(program.hash().to_string()),
        cycles: Some(trace.get_trace_len() as u32),
        error: None,
        compilation_time_ms: Some(compilation_time),
        execution_time_ms: Some(execution_time),
        total_time_ms: Some(total_time),
    }
}

pub fn generate_proof_impl(program: &str, inputs_json: Option<&str>) -> ProofResult {
    let total_start = Instant::now();
    let source_manager = Arc::new(DefaultSourceManager::default());

    let compilation_start = Instant::now();
    let assembler = match Assembler::default()
        .with_debug_mode(true)
        .with_static_library(StdLibrary::default())
    {
        Ok(asm) => asm,
        Err(e) => {
            return ProofResult {
                success: false,
                proof_bytes: None,
                program_hash: None,
                stack_outputs: None,
                error: Some(format!("Failed to configure assembler: {e}")),
                compilation_time_ms: None,
                proving_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    // Parse stack inputs
    let stack_inputs = if let Some(inputs_str) = inputs_json {
        match parse_stack_inputs(inputs_str) {
            Ok(inputs) => inputs,
            Err(e) => {
                return ProofResult {
                    success: false,
                    proof_bytes: None,
                    program_hash: None,
                    stack_outputs: None,
                    error: Some(e),
                    compilation_time_ms: None,
                    proving_time_ms: None,
                    total_time_ms: Some(total_start.elapsed().as_millis() as f64),
                }
            }
        }
    } else {
        StackInputs::default()
    };

    // Assemble the program
    let program = match assembler.assemble_program(program) {
        Ok(prog) => prog,
        Err(e) => {
            return ProofResult {
                success: false,
                proof_bytes: None,
                program_hash: None,
                stack_outputs: None,
                error: Some(format!("Assembly error: {e}")),
                compilation_time_ms: Some(compilation_start.elapsed().as_millis() as f64),
                proving_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    let compilation_time = compilation_start.elapsed().as_millis() as f64;

    // Generate proof
    let proving_start = Instant::now();
    let (outputs, proof) = match prove(
        &program,
        stack_inputs,
        AdviceInputs::default(),
        &mut DefaultHost::default(),
        ProvingOptions::default(),
        source_manager,
    ) {
        Ok(result) => result,
        Err(e) => {
            return ProofResult {
                success: false,
                proof_bytes: None,
                program_hash: None,
                stack_outputs: None,
                error: Some(format!("Proving error: {e}")),
                compilation_time_ms: Some(compilation_time),
                proving_time_ms: None,
                total_time_ms: Some(total_start.elapsed().as_millis() as f64),
            }
        }
    };

    let proving_time = proving_start.elapsed().as_millis() as f64;
    let total_time = total_start.elapsed().as_millis() as f64;

    let stack_outputs: Vec<String> = outputs
        .first()
        .iter()
        .take(16)
        .map(|f| f.to_string())
        .collect();

    ProofResult {
        success: true,
        proof_bytes: Some(proof.to_bytes()),
        program_hash: Some(program.hash().to_string()),
        stack_outputs: Some(stack_outputs),
        error: None,
        compilation_time_ms: Some(compilation_time),
        proving_time_ms: Some(proving_time),
        total_time_ms: Some(total_time),
    }
}
