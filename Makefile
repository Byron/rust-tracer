.phony:  image

all: rtrace
rtrace:
	cargo build --release
image: rtrace
	time ./target/release/rtrace --samples-per-pixel=4 --width=1024 --height=768 out.tga