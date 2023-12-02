all: release

release:
	cargo build --all-features --release

debug:
	cargo build --all-features

clean:
	rm *.profraw

