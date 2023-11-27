all : rust/target/debug/blue
	cd rust ; cargo run
rust/target/debug/blue : rust/src/main.rs
	cd rust ; cargo build
clean :
	cd rust ; cargo clean
