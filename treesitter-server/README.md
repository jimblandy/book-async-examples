# Rust parsing server

This crate implements an HTTP server that accepts requests containing
Rust source code and replies with its parse tree.

To submit some Rust code to parse, make a `GET` request to `v1/parse`,
with a body like this:

    {
        "source": "fn f() -> i32 { 42 }",
        "timeout_seconds": 10
    }

The response is a tree-sitter style S-expression representing the parse tree.

To try this out, start the server in one terminal:

    $ cargo run
       Compiling async-treesitter v0.1.0 (.../async-treesitter)
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.42s
         Running `.../target/debug/async-treesitter`
    [2025-11-11T22:54:28Z INFO  async_treesitter] Serving async treesitter API at 0.0.0.0:3000
    [2025-11-11T22:54:28Z INFO  warp::server] Server::run; addr=0.0.0.0:3000
    [2025-11-11T22:54:28Z INFO  warp::server] listening on http://0.0.0.0:3000

Then send a request using `curl`:

    $ curl http://localhost:3000/v1/parse \
           --json '{"source": "fn f() -> i32 { 42 }", "timeout_seconds": 10 }'
    (source_file (function_item name: (identifier) parameters: (parameters) return_type: (primitive_type) body: (block (integer_literal))))
    $ 

