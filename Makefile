.phony:  image

all: rtrace
rtrace:
	cargo build --release
image: rtrace
	time ./target/release/rtrace > out.tga