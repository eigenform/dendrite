cmake_minimum_required(VERSION 3.28)
project(dendrite)

add_library(dendrite SHARED src/dendrite.c)
find_package(DynamoRIO)
if (NOT DynamoRIO_FOUND)
  message(FATAL_ERROR "DynamoRIO package required to build")
endif(NOT DynamoRIO_FOUND)
configure_DynamoRIO_client(dendrite)
use_DynamoRIO_extension(dendrite drmgr)
use_DynamoRIO_extension(dendrite drreg)
use_DynamoRIO_extension(dendrite drx)

