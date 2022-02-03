schema:
	@find contracts/* -maxdepth 0 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo schema" \;

build:
	@./build_release.sh

test:
	@cargo test