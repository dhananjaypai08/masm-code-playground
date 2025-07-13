use crate::client::{ExecutionRequest, ExecutionResult, ProofResult, execute_program_impl, generate_proof_impl};

#[cfg(feature = "web_server")]
use axum::{
    extract::Json,
    http::Method,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};

#[cfg(feature = "web_server")]
use tower_http::cors::{Any, CorsLayer};

#[cfg(feature = "web_server")]
use serde_json::json;

#[cfg(feature = "web_server")]
use std::net::SocketAddr;

#[cfg(feature = "web_server")]
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/api/examples", get(examples_handler))
        .route("/api/execute", post(execute_handler))
        .route("/api/prove", post(prove_handler))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Miden VM API Server starting on http://0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(feature = "web_server")]
async fn health_handler() -> ResponseJson<serde_json::Value> {
    ResponseJson(json!({
        "status": "healthy",
        "service": "miden-vm-api",
        "version": "0.1.0"
    }))
}

#[cfg(feature = "web_server")]
async fn examples_handler() -> ResponseJson<serde_json::Value> {
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
        ("Basic Addition", "# Adds 3 + 5 = 8\nbegin\n    push.3\n    push.5\n    add\nend"),
        ("Input Stack Demo", "# Adds two inputs: Try with [\"10\", \"20\"] → 30\nbegin\n    add\nend"),
        ("Fibonacci Numbers", "# Computes F(8) using repeat loop\n\
        # Input: { \"operand_stack\": [\"1\"] }\n\
        begin\n
        # This code computes 69th Fibonacci number\n
        repeat.68\n
            swap dup.1 add\n
            end\n
        end"),
        ("Prime Generator", prime_generator),
        ("Conditional Logic", "# Keeps larger of 15 and 10 (result: 15)\nbegin\n    push.15\n    push.10\n    dup.1 gt\n    if.true\n        swap\n    end\n    drop\nend"),
        ("Memory Operations", "# Stores 42 and 100 in memory, sums them (result: 142)\nbegin\n    push.42 push.0 mem_store\n    push.100 push.1 mem_store\n    push.0 mem_load\n    push.1 mem_load\n    add\nend"),
        ("Stack Manipulation", "# Leaves [3,3] on the stack\nbegin\n    push.1 push.2 push.3 push.4\n    swap.2\n    drop\n    dup\n    swap.2\n    drop\n    drop\nend"),
        ("Counter with Input", "# Adds 5 to input. Try [\"7\"] → 12\nbegin\n    push.5\n    add\nend"),
    ];
    
    ResponseJson(json!(examples))
}

#[cfg(feature = "web_server")]
async fn execute_handler(Json(payload): Json<ExecutionRequest>) -> ResponseJson<ExecutionResult> {
    let inputs_json = payload.inputs.as_ref().map(|v| v.to_string());
    let result = execute_program_impl(&payload.program, inputs_json.as_deref());
    ResponseJson(result)
}

#[cfg(feature = "web_server")]
async fn prove_handler(Json(payload): Json<ExecutionRequest>) -> ResponseJson<ProofResult> {
    let inputs_json = payload.inputs.as_ref().map(|v| v.to_string());
    let result = generate_proof_impl(&payload.program, inputs_json.as_deref());
    ResponseJson(result)
}