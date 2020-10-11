 ./target/release/substrate --base-path /tmp/alice \
   --chain local --alice \
   --port 30333 --ws-port 9944 --rpc-port 9933 \
   --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
   --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
   --validator \
   --cert=scripts/alice.der \
<<<<<<< HEAD
   --anchors=scripts/ca.der

./target/release/substrate  --base-path /tmp/bob \
     --chain local  --bob \
     --port 30334  --ws-port 9945  --rpc-port 9934 \
     --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
     --validator \
     --cert=scripts/bob.der \
     --anchors=scripts/ca.der
     --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
=======
   --anchors=anchors
>>>>>>> 03a8f70fb546e469cc0cd39af69b585576a58ffa

./target/release/substrate  --base-path /tmp/bob \
     --chain local  --bob \
     --port 30334  --ws-port 9945  --rpc-port 9934 \
     --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
     --validator \
     --cert=/Users/junius/github/rust/rustls/test-ca/eddsa/end.der \
<<<<<<< HEAD
     --anchors=scripts/ca.der
=======
     --anchors=anchors
>>>>>>> 03a8f70fb546e469cc0cd39af69b585576a58ffa
     --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp


export RUST_LOG=info,libp2p=debug


target/release/substrate --dev --cert=scripts/alice.der --anchors=scripts/ca.der

wrong certificate.
/Users/junius/github/rust/rustls/test-ca/eddsa/end.der


<<<<<<< HEAD
=======

>>>>>>> 03a8f70fb546e469cc0cd39af69b585576a58ffa
# db location. 
/Users/junius/Library/Application Support/substrate/chains/dev/db

