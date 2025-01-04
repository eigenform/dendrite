# dendrite

A Rust library with examples and other tools for exploring the accuracy of 
branch prediction strategies over traces. 

## Building Instrumentation

This project depends on using some kind of binary instrumentation to 
capture traces. Currently, only x86 platforms are supported. 

See [pin/](./pin) for instructions on how to build the `dendrite` plugin for 
Intel Pin.

> **NOTE**: 
> See [dynamorio/](./dynamorio) for instructions on how to build the plugin
  for DynamoRIO (**warning: will be deprecated and replaced by Pin in the near 
  future**)

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

- Tables, saturating counters, a naive perceptron
- Trivial predictors (always-taken, never-taken, random, etc)
- A configurable TAGE ("TAgged GEometric length") predictor

Sometime in the future, this project will also include components for 
reasoning about target predictors (ie. branch target buffers, return-address
predictors, indirect predictors, etc). 

## Binaries

[dendrite/src/bin](./dendrite/src/bin) contains utilities, examples, and 
other experiments: 

- `read-trace`: Print the contents of a trace file
- `analyze-trace`: Print statistics about branch behavior in a trace
- `evaluate`: Evaluate built-in predictors against a trace 

Typical usage looks something like this: 

```
# Use Intel Pin to write a trace
$ /opt/intel-pin/pin -t pin/obj-intel64/dendrite.so -o /tmp/trace.bin -- ls
...

# Print the raw entries in the trace
$ cargo run --release --bin read-trace -- /tmp/trace.bin
...
```

