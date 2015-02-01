.phony:  image

all: gotrace
gotrace: gotrace.go
	go build -ldflags="-w -s" gotrace.go
clean:
	rm -f out.tga
image: gotrace
	time ./gotrace