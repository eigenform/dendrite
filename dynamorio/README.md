
[DynamoRIO](https://github.com/DynamoRIO/dynamorio) is a library for creating 
binary instrumentation tools. The client library in this directory can be used 
to generate traces for evaluating models built with `dendrite`. 

# Usage

```
# Build the client library 'libdendrite.so'
$ ./build.sh
...

# Generate a trace by instrumenting some binary. 
$ /opt/dynamorio/bin64/drrun -c build/libdendrite.so -- ls
...

# Trace data is written to a binary file in /tmp
$ xxd /tmp/dendrite.ls.04782.0000.bin | less
...
```

The trace format is flat binary data (see [dendrite.h](./src/dendrite.h)) 
without any sort of compression. Keep in mind that trace files can become 
*very large*, even for programs that are very short-running without 
instrumentation. 

