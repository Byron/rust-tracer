G=6g
L=6l

all: cpptrace
.phony : image

clean:
	rm -f cpptrace out.tga

cpptrace: ray.cpp
	# Using g++ with -O3 bossts performance by 100% and more , go doesnt 
	# have that yet
	# g++ -O3 ray.cpp -o cpptrace
	g++ ray.cpp -o cpptrace

image : cpptrace
	time ./cpptrace > out.tga
