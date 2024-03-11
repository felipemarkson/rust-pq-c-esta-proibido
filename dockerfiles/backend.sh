set -ex

_term() { 
  echo "Caught EXIT signal!"
  pkill -TERM backend
  exit 0
}

trap _term SIGTERM SIGINT
backend $PORT &
wait $!