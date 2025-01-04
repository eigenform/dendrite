

[Intel Pin](https://www.intel.com/content/www/us/en/developer/articles/tool/pin-a-dynamic-binary-instrumentation-tool.html) is a library for creating binary 
instrumentation tools. This project assumes you've installed Intel Pin 3.31. 

The plugin in this directory is used to create traces for evaluating models 
built with `dendrite`.

# Building

First, you'll need to ensure that `$PIN_ROOT` is set in your shell. 
The makefiles depend on this. 
For example, my `~/.bash_profile` contains the following: 

```
export PIN_ROOT=/opt/intel-pin
```

Afterwards, you can build `dendrite.so` by running `make`.

# Usage

By default, the tool writes a binary trace to `/tmp/trace.bin`. 
Some usage examples:

```
# Write a trace for the `ls` binary to `/tmp/output.bin`
$ /opt/intel-pin/pin -t ./obj-intel64/dendrite.so -o /tmp/output.bin -- /bin/ls
...

```

## Trace Format

For now, traces only record information about control-flow instructions.
All traces are written to `/tmp/`. 

Traces are flat binary data (see [dendrite.h](./src/dendrite.h)) stored 
without any sort of compression. Keep in mind that trace files can become 
*very large*, even for programs that are very short-running without any
instrumentation. 

