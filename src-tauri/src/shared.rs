// src-tauri/src/shared.rs
use std::sync::Arc;
use miden_vm::{
    assembly::DefaultSourceManager, 
    execute, prove, utils::Serializable, 
    AdviceInputs, Assembler, DefaultHost, Program, ProvingOptions, StackInputs
};
use miden_processor::ExecutionOptions;
use miden_stdlib::StdLibrary;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize)]
pub struct ProofResult {
    pub success: bool,
    pub proof_bytes: Option<Vec<u8>>,
    pub program_hash: Option<String>,
    pub stack_outputs: Option<Vec<String>>,
    pub error: Option<String>,
}

pub fn get_example_programs() -> Vec<(&'static str, &'static str)> {
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

    vec![
        ("Basic Addition", "begin push.3 push.5 add end"),
        ("Fibonacci", "begin push.1 push.1 push.8 repeat.6 dup.1 add swap end drop"),
        ("Simple Loop", "begin push.0 push.10 repeat.10 dup add.1 end drop end"),
        ("Conditional", "begin push.5 push.3 dup.1 gt if.true swap end drop end"),
        ("Stack Manipulation", "begin push.1 push.2 push.3 push.4 swap.2 drop dup end"),
        ("Memory Operations", "begin push.42 push.0 mem_store push.0 mem_load end"),
        ("Prime Generator (Requires Input)", prime_generator),
    ]
}

pub fn parse_stack_inputs(inputs_json: &str) -> Result<StackInputs, String> {
    let parsed: Value = serde_json::from_str(inputs_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    let mut inputs = Vec::new();
    
    if let Some(operand_stack) = parsed.get("operand_stack") {
        if let Some(stack_array) = operand_stack.as_array() {
            for item in stack_array {
                if let Some(val_str) = item.as_str() {
                    let val: u64 = val_str.parse()
                        .map_err(|e| format!("Invalid number '{}': {}", val_str, e))?;
                    inputs.push(val);
                } else if let Some(val_num) = item.as_u64() {
                    inputs.push(val_num);
                }
            }
        }
    }
    
    // Reverse because stack inputs are pushed in reverse order
    inputs.reverse();
    
    StackInputs::try_from_ints(inputs)
        .map_err(|e| format!("Failed to create stack inputs: {}", e))
}

pub fn execute_program_impl(program: &str, inputs_json: Option<&str>) -> ExecutionResult {
    let source_manager = Arc::new(DefaultSourceManager::default());
    
    // Create assembler with source manager and standard library
    let mut assembler = match Assembler::default()
        .with_debug_mode(true)
        .with_static_library(&StdLibrary::default()) {
        Ok(asm) => asm,
        Err(e) => return ExecutionResult {
            success: false,
            stack_outputs: None,
            program_hash: None,
            cycles: None,
            error: Some(format!("Failed to configure assembler: {}", e)),
        }
    };
    
    // Parse and set up stack inputs
    let stack_inputs = if let Some(inputs_str) = inputs_json {
        match parse_stack_inputs(inputs_str) {
            Ok(inputs) => inputs,
            Err(e) => return ExecutionResult {
                success: false,
                stack_outputs: None,
                program_hash: None,
                cycles: None,
                error: Some(e),
            }
        }
    } else {
        StackInputs::default()
    };
    
    // Assemble the program
    let program = match assembler.assemble_program(program) {
        Ok(prog) => prog,
        Err(e) => return ExecutionResult {
            success: false,
            stack_outputs: None,
            program_hash: None,
            cycles: None,
            error: Some(format!("Assembly error: {}", e)),
        }
    };
    
    let advice_inputs = AdviceInputs::default();
    let mut host = DefaultHost::default();
    let exec_options = ExecutionOptions::default();
    
    // Execute the program
    let trace = match execute(
        &program, 
        stack_inputs.clone(), 
        advice_inputs.clone(), 
        &mut host, 
        exec_options, 
        source_manager.clone()
    ) {
        Ok(trace) => trace,
        Err(e) => return ExecutionResult {
            success: false,
            stack_outputs: None,
            program_hash: None,
            cycles: None,
            error: Some(format!("Execution error: {}", e)),
        }
    };
    
    // Get stack outputs (show more elements)
    let stack_outputs: Vec<String> = trace.stack_outputs()
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
    }
}

pub fn generate_proof_impl(program: &str, inputs_json: Option<&str>) -> ProofResult {
    let source_manager = Arc::new(DefaultSourceManager::default());
    
    let mut assembler = match Assembler::default()
        .with_debug_mode(true)
        .with_static_library(&StdLibrary::default()) {
        Ok(asm) => asm,
        Err(e) => return ProofResult {
            success: false,
            proof_bytes: None,
            program_hash: None,
            stack_outputs: None,
            error: Some(format!("Failed to configure assembler: {}", e)),
        }
    };
    
    // Parse stack inputs
    let stack_inputs = if let Some(inputs_str) = inputs_json {
        match parse_stack_inputs(inputs_str) {
            Ok(inputs) => inputs,
            Err(e) => return ProofResult {
                success: false,
                proof_bytes: None,
                program_hash: None,
                stack_outputs: None,
                error: Some(e),
            }
        }
    } else {
        StackInputs::default()
    };
    
    // Assemble the program
    let program = match assembler.assemble_program(program) {
        Ok(prog) => prog,
        Err(e) => return ProofResult {
            success: false,
            proof_bytes: None,
            program_hash: None,
            stack_outputs: None,
            error: Some(format!("Assembly error: {}", e)),
        }
    };
    
    // Generate proof
    let (outputs, proof) = match prove(
        &program,
        stack_inputs,
        AdviceInputs::default(),
        &mut DefaultHost::default(),
        ProvingOptions::default(),
        source_manager,
    ) {
        Ok(result) => result,
        Err(e) => return ProofResult {
            success: false,
            proof_bytes: None,
            program_hash: None,
            stack_outputs: None,
            error: Some(format!("Proving error: {}", e)),
        }
    };
    
    let stack_outputs: Vec<String> = outputs.first()
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
    }
}