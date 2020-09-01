 ./target/release/substrate --base-path /tmp/alice \
   --chain local --alice \
   --port 30333 --ws-port 9944 --rpc-port 9933 \
   --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
   --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
   --validator \
   --cert=/Users/junius/github/junius/asia-rust-libp2p/scripts/alice.cert \
   --anchors=anchors

./target/release/substrate  --base-path /tmp/bob \
     --chain local  --bob \
     --port 30334  --ws-port 9945  --rpc-port 9934 \
     --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
     --validator \
     --cert=/Users/junius/github/junius/asia-rust-libp2p/scripts/bob.cert \
     --anchors=anchors
     --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp


