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
rm -f client_*.db
rm -f log.log


cargo run --release --bin database &
cargo run --release --bin backend 8000 &
cargo run --release --bin backend 8001 &
cargo run --release --bin httpserver 8000 8001  > log.log &
wait $!