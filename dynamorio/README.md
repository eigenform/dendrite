
If you have DynamoRIO, you can use this to generate traces. 
In my case, I'm doing something like this:

```
# Build the client library
$ ./build.sh
...

# Generate a trace by instrumenting and running '/usr/bin/ls'
$ /opt/dynamorio/bin64/drrun -c build/libdendrite.so -- ls
```

