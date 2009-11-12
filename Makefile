G=6g
L=6l

all: gotrace

clean:
	rm -f gotrace gotrace.6 out.ppm out.tga

gotrace: gotrace.6
	$(L) -o gotrace gotrace.6

%.6:	%.go
	$(G) $(F) $<

