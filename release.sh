set -e
ENV=$1
DIR="$(dirname "$0")"
CLIENT_DIR="$DIR/crates/seven-oz-client"
API_DIR="$DIR/crates/seven-oz-loyalty"
DIST="$API_DIR/assets"

echo "Releasing to $ENV - $DIST"

rm -f $DIST/*
trunk build --release --features $ENV --dist $DIST $CLIENT_DIR/index.html

set +e
cargo shuttle stop --name 7oz-loyalty
cargo shuttle deploy --allow-dirty --name 7oz-loyalty --working-directory $API_DIR