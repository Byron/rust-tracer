G=6g
L=6l

all: gotrace
.phony : image

clean:
	rm -f gotrace out.tga

gotrace: ray.cpp
	g++ -O3 -ffast-math -funroll-all-loops ray.cpp -o gotrace

image : gotrace
	time ./gotrace > out.tga
