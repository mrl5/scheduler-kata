#!/bin/bash

time for x in $(seq 20); do
    just test-api &
    just test-api &
    just test-api &
    just test-api &
    just test-api &
    wait && sleep 5
done
