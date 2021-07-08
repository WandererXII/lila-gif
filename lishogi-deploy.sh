#!/bin/sh -e
cargo +stable build --release --target x86_64-unknown-linux-musl
ssh "ubuntu@$1.lishogi.ovh" mv /usr/local/bin/lishogi-gif /usr/local/bin/lishogi-gif.bak || (echo "first deploy on this server? set up service and comment out this line" && false)
scp ./target/x86_64-unknown-linux-musl/release/lishogi-gif "ubuntu@$1.lishogi.ovh":/usr/local/bin/lishogi-gif
ssh "ubuntu@$1.lishogi.ovh" systemctl restart lishogi-gif
