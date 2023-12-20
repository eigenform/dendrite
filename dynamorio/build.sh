#!/bin/sh

if [ ! -e ./build ]; then
	mkdir build
fi

pushd build
cmake ../
make 
popd
