[![title](http://img.youtube.com/vi/uhYRveqF27U/0.jpg)](http://www.youtube.com/playlist?list=PLMHbQxe1e9MlR80JVZCa0uJf9cz_PxlCY)

## Trace Quest 5: Rise of an Empire

Following cinematic success stories such as [The SHA1 Performance Quest](https://www.youtube.com/playlist?list=PLMHbQxe1e9MnDKy7FKXZwMJ6t_RCxpHqD), there is yet another quest for performance, seeking
out benchmark results that no man has seen before !

It's stardate 20.15.01.31, in a universe ruled by nimble Gophers and ancient Ceepeporg, a universe 
where the production of RaySpheres™ is the only goal, the faster, the better.
Meet the Rustaceans, a young species, and follow them on their curious quest for independence and for producing the most RaySperes in the universe.

## How to run

Make sure you have gcc installed as well as go for the respective version. Then it's the following to produce images and
time them.
Please note that they will only use one core by default - the cpp version doesn't impelement multi-threading.

```bash
make -C src/go image
make -C src/cpp image

# Use more cores with go implementation to witness speedup
GOMAXPROCS=4 src/go image
```

## Original Credits

The `go` based raytracer was originally written by Jack Palevich. For more information, see http://grammerjack.blogspot.com/2009/11/multi-threaded-go-raytracer.html.

