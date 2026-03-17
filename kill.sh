#!/bin/bash

kill $(lsof -i:50051 -t)
kill $(lsof -i:5430 -t)
kill $(lsof -i:5431 -t)
kill $(lsof -i:1234 -t)
kill $(lsof -i:1235 -t)