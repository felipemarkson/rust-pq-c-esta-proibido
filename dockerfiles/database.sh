set -ex

_term() { 
  echo "Caught EXIT signal!"
  pkill -TERM database
  exit 0
}

trap _term SIGTERM SIGINT
database &
wait $!