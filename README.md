[![title](http://img.youtube.com/vi/uhYRveqF27U/0.jpg)](http://www.youtube.com/playlist?list=PLMHbQxe1e9MlR80JVZCa0uJf9cz_PxlCY)

## Trace Quest 5: Rise of an Empire

Following cinematic success stories such as [The SHA1 Performance Quest](https://www.youtube.com/playlist?list=PLMHbQxe1e9MnDKy7FKXZwMJ6t_RCxpHqD), there is yet another quest for performance, seeking
out benchmark results that no man has seen before !

It's stardate 20.15.01.31, in a universe ruled by nimble Gophers and ancient Ceepeporg, a universe 
where the production of RaySpheres™ is the only goal, the faster, the better.
Meet the Rustaceans, a young species, and follow them on their curious quest for independence and for producing the most RaySperes in the universe.

## How to run

Generating images is as easy as running make in the respective source folder. We assume you have rust, go and gcc installed and in your PATH.

```bash
# rust
make image
# go
make -C src/go image
# c++
make -C src/cpp image

# Use more cores with go implementation to witness speedup
GOMAXPROCS=4 make src/go image
# Same with rust
RTRACEMAXPROCS=4 make image
```

## Season 1 Conclusion

![rtrace-image](https://raw.githubusercontent.com/Byron/rust-tracer/master/src/img/rtrace-output.png)

Even though the rustacean RaySpheres are the prettiest thanks to an improved shading algorithm, on a single core we are not fastest. For some reason, **C++ is 7 percent faster** even though it is inefficient when creating the scene of more than 20.000 spheres and even though it uses virtual method calls.

Only when rust enters multi-threaded rendering mode, it is the fastest in town, as C++ doesn't implement multi-threading in this case.

Not to forget, go, which is far behind being about three times slower, no matter what.

## Season 2 Preview

Things to do, in season two ... 

* Let multiple threads share a single image buffer, and handle the resulting unsafe code accordingly.
 * Currently, we use more memory than needed as every thread owns its own 64x64x4 u8 byte buffer, which is then written into a full-sized one resembling the final frame.
* Use piston window for real-time rendering visualization (in addition to image output)
* Implement interactive rendering with undersampling and simple camera controls.
* Use the mouse-pointer to focus rendering on buckets underneath

## Lessons Learned

* Instead of parameterizing primitive number types in generics, use a type alias instead. That way, the code is simpler overall, and you may use number literals in your code. This is viable if you just want to test how different number types will affect your runtime performance.
* When using multi-threading, everything used by a thread needs to have static lifetime. To assure that, you can
  * create resources within the thread, handle them and send a particular result through a channel.
  * pass in read-only shared resources through an `Arc`. The latter is a reference counted item on the heap.
  * pass writable resources through an `Arc<Mutex<R>>` to allow synchronized access.
  * *Yes, it's not currently possible to signal a resource is safe for use if allocated on a stack and protected by guards that will assure the resource remains alive during the thread runtime.*

## Original Credits

The `go` based raytracer was originally written by Jack Palevich. For more information, see http://grammerjack.blogspot.com/2009/11/multi-threaded-go-raytracer.html.

