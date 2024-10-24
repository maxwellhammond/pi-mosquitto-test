buildd:
	cargo build --target=aarch64-unknown-linux-gnu
	scp target/aarch64-unknown-linux-gnu/debug/rusty-pi amtadmin@10.10.40.145:/home/amtadmin

buildr:
	cargo build --release --target=aarch64-unknown-linux-gnu