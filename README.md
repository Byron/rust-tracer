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
GOMAXPROCS=4 make src/go image
```

## Season 1 Conclusion



## Lessons Learned

* Instead of parameterizing primitive number types in generics, use a type alias instead. That way, the code is simpler overall, and you may use number literals in your code. This is viable if you just want to test how different number types will affect your runtime performance.
* When using multi-threading, everything used by a thread needs to have static lifetime. To assure that, you can
  * create resources within the thread, handle them and send a particular result through a channel.
  * pass in read-only shared resources through an `Arc`. The latter is a reference counted item on the heap.
  * pass writable resources through an `Arc<Mutex<R>>` to allow synchronized access.
  * *Yes, it's not currently possible to signal a resource is safe for use if allocated on a stack and protected by guards that will assure the resource remains alive during the thread runtime.*

## Original Credits

The `go` based raytracer was originally written by Jack Palevich. For more information, see http://grammerjack.blogspot.com/2009/11/multi-threaded-go-raytracer.html.

