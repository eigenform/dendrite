
> **NOTE**: 
> **Use of the DynamoRIO library will be deprecated and replaced by the 
> Intel Pin library in the near future.**

[DynamoRIO](https://github.com/DynamoRIO/dynamorio) is a library for creating 
binary instrumentation tools. After installing DynamoRIO, the client library 
in this directory can be used to generate traces for evaluating models built 
with `dendrite`.

```
# Build the client library 'libdendrite.so'
$ ./build.sh
...

# Generate a trace by instrumenting some binary. 
$ /opt/dynamorio/bin64/drrun -c build/libdendrite.so -- ls
...

# The trace data is written to a binary file in /tmp/. 
$ xxd /tmp/dendrite.ls.04782.0000.bin | less
...
```

# Trace Format

For now, traces only record information about control-flow instructions.
All traces are written to `/tmp/`. 

Traces are flat binary data (see [dendrite.h](./src/dendrite.h)) stored 
without any sort of compression. Keep in mind that trace files can become 
*very large*, even for programs that are very short-running without any
instrumentation. 

