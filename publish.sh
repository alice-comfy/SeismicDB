pushd crates

pushd sdb-core
cargo publish --allow-dirty
popd

pushd sdb-server-core
cargo publish --allow-dirty
popd

pushd sdb-cli
cargo publish --allow-dirty
popd

popd

cargo publish --allow-dirty
