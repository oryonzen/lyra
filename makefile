all:
	cargo run

clean:
	cmake -E rm -rf build
	cargo clean
