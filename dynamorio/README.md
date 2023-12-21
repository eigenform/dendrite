
[DynamoRIO](https://github.com/DynamoRIO/dynamorio) is a library for creating 
binary instrumentation tools. This client can be used to generate traces 
that we can use to evaluate models built with `dendrite`. 

For example:

```
# Build the client library 'libdendrite.so'
$ ./build.sh
...

# Generate a trace by instrumenting some binary
$ /opt/dynamorio/bin64/drrun -c build/libdendrite.so -- ls
...

# Trace data is written to /tmp
$ xxd /tmp/dendrite.ls.04782.0000.bin | less
...
```

