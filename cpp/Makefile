G=6g
L=6l

all: cpptrace
.phony : image

clean:
	rm -f cpptrace out.tga

cpptrace: ray.cpp
	g++ -O3 -ffast-math -funroll-all-loops ray.cpp -o cpptrace

image : cpptrace
	time ./cpptrace > out.tga
