# dendrite

A Rust library with examples and other tools for exploring the accuracy of 
branch prediction strategies over DynamoRIO traces. 

## Building the DynamoRIO Plugin

This project depends on [DynamoRIO](https://github.com/DynamoRIO/dynamorio).
Program traces are collected by running an x86 executable with a simple plugin.
See [dynamorio/](./dynamorio) for instructions on how to build the plugin.

## Building this Project

You can build the library and all included binaries with the following: 

```
$ cargo build --release
... 
```

## Predictors and Building Blocks

This library also includes some [potentially incorrect and/or unfinished] 
implementations of different branch predictors and common components that
you can use to implement them, ie. 

- Tables, saturating counters, perceptrons
- Trivial predictors (always-taken, never-taken, random, etc)
- A configurable TAGE ("TAgged GEometric length") predictor

## Binaries

[dynamorio/src/bin](./dynamorio/src/bin) contains utilities, examples, and 
other experiments: 

- `read-trace`: Print the contents of a trace file
- `analyze-trace`: Print statistics about branch behavior in a trace
- `evaluate`: Evaluate built-in predictors against a trace 

Typical usage looks something like this: 

```
# Use `drrun` to write a trace (to the /tmp/ directory)
$ /opt/dynamorio/bin64/drrun -c build/libdendrite.so -- ls
...

# Print the raw entries in the trace
$ cargo run --release --bin read-trace -- /tmp/dendrite.ls.04782.0000.bin
...
```


