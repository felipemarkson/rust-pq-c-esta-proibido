set -ex

_term() { 
  echo "Caught EXIT signal!"
  pkill -TERM httpserver
  exit 0
}

trap _term SIGTERM SIGINT
httpserver 8000 8001 &
wait $!