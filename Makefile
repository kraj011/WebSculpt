build:
	cargo build

run: build
	MTL_HUD_ENABLED=1 ./target/debug/web_sculpt

