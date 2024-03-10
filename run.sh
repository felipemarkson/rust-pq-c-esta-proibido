#!/bin/sh

set -ex


_term() { 
  echo "Caught EXIT signal!"
  pkill -TERM database
  pkill -TERM backend
  pkill -TERM httpserver
  exit 0
}

trap _term SIGTERM SIGINT


cargo run --release --bin database &
db_pid=$!
cargo run --release --bin backend &
backend_pid=$!
cargo run --release --bin httpserver &
server_pid=$!
echo "$db_pid $backend_pid $server_pid"
wait $server_pid