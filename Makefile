.phony:  image

all: rtrace
rtrace: target/release/rtrace
	cargo build --release
image: rtrace
	time ./target/release/rtrace > out.tga