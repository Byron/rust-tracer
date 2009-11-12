// Adapted from http://www.ffconsultancy.com/languages/ray_tracer/comparison.html

package main
import fmt "fmt"
import io "io"
import os "os"
import math "math"

var infinity float = float(math.Inf(1));
var delta float = float(math.Sqrt(1.19209E-07)); // sqrt(float_epsilon)

func sqrtf(a float) float {
    return float(math.Sqrt(float64(a)));
}

type Vec3 struct {
    x, y, z float;
}

func vec3add(a Vec3, b Vec3) Vec3 {
    a.x += b.x;
    a.y += b.y;
    a.z += b.z;
    return a;
}

func vec3sub(a Vec3, b Vec3) Vec3 {
    a.x -= b.x;
    a.y -= b.y;
    a.z -= b.z;
    return a;
}

func vec3mulf(a Vec3, b float) Vec3 {
    a.x *= b;
    a.y *= b;
    a.z *= b;
    return a;
}

func vec3dot(a Vec3, b Vec3) float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

func normalize(a Vec3) Vec3 {
    return vec3mulf(a, 1.0 / sqrtf(vec3dot(a,a)));
}

var backgroundColor Vec3 = Vec3{0.1,0.1,0.1};
var diffuseSphereColor Vec3 = Vec3{0.0, 0.7, 0.0};
var ambientSphereColor Vec3 = Vec3{0.2, 0.3, 0.2};

type Sphere struct {
    center Vec3;
    radius float;
}

type Hit struct {
    distance float;
    pos Vec3;
    sphere *Sphere;
}

type Ray struct {
    orig, dir Vec3;
}

type Geometry interface {
    Intersect(h Hit, r Ray) Hit;
    Print(); // Temporary until fmt handles interfaces.
}

func (s *Sphere) RaySphere(r Ray) float {
    v := vec3sub(s.center, r.orig);
    b := vec3dot(v, r.dir);
    disc := b*b - vec3dot(v, v) + s.radius * s.radius;
    if disc < 0.0 {
        return infinity;
    }
    d := sqrtf(disc);
    t2 := b + d;
    if t2 < 0.0 {
        return infinity;
    }
    t1 := b - d;
    if t1 > 0.0 {
        return t1;
    }
    return t2;
}

func (s *Sphere) Intersect(h Hit, r Ray) Hit {
    lambda := s.RaySphere(r);
    if lambda >= h.distance {
        return h;
    }
    return Hit{lambda, normalize(vec3add(r.orig, vec3sub(vec3mulf(r.dir, lambda), s.center))), s};
}

func (s *Sphere) Print() {
    fmt.Println("Sphere:",*s);
}

type Group struct {
    bound Sphere;
    children []Geometry;
}

func (g *Group) Print() {
    fmt.Print("Group:");
    g.bound.Print();
    for i := 0; i < len(g.children); i++ {
        fmt.Print("  ");
        g.children[i].Print();
    }
}

func (g *Group) Intersect(h Hit, r Ray) Hit {
    l := g.bound.RaySphere(r);
    if l >= h.distance {
        return h;
    }
    hit2 := h;
    for i := 0; i < len(g.children); i++ {
      hit2 = g.children[i].Intersect(hit2, r);
    }
    return hit2;
}

func NewGroup(bound Sphere, children []Geometry) *Group {
    g := new(Group);
    g.bound = bound;
    g.children = children;
    return g;
}

func intersect(r Ray, s Geometry) Hit {
    return s.Intersect(Hit{infinity, Vec3{0, 0, 0}, nil}, r);
}

type Scene struct {
    light Vec3;
    g Geometry;
}

func createScene(light Vec3, g Geometry) *Scene {
    scene := new(Scene);
    scene.light = light;
    scene.g = g;
    return scene;
}

func (s *Scene) rayTrace(r Ray) Vec3 {
    h := intersect(r, s.g);
    if h.distance == infinity {
        return backgroundColor;
    }
    g := vec3dot(h.pos, s.light);
    if g >= 0.0 {
        // The hit intersection is in shadow
        return ambientSphereColor;
    }
    p := vec3add(r.orig, vec3add(vec3mulf(r.dir, h.distance), vec3mulf(h.pos, delta)));
    if intersect(Ray{p, vec3mulf(s.light, -1.0)}, s.g).distance < infinity {
        // There's an object between us and the light.
        return ambientSphereColor;
    }
    litColor := vec3mulf(diffuseSphereColor, -g);
    totalColor := vec3add(ambientSphereColor, litColor);
    return totalColor;
}

func createSpherePyramid(level int, c Vec3, r float) Geometry {
    s := new(Sphere);
    s.center = c;
    s.radius = r;
    if level == 1 {
        return s;
    }
    children := make([]Geometry, 5);
    i := 0;
    children[i] = s;
    i++;
    rn := 3.0*r/sqrtf(12.0);
    for dz := -1; dz<=1; dz+=2 {
        for dx := -1; dx<=1; dx+=2 {
            newc := vec3add(c, vec3mulf(Vec3{float(dx), 1.0, float(dz)}, rn));
            children[i] = createSpherePyramid(level-1, newc, r * 0.5);
            i++;
        }
    }
    return NewGroup(Sphere{c, 3*r}, children);
}

type Texture struct {
    w, h int;
    buf []byte;
}

func NewTexture(w int, h int) *Texture {
    t := new(Texture);
    t.w = w;
    t.h = h;
    t.buf = make([]byte, w * h * 4);
    return t;
}

func (t *Texture) WritePPM(w io.Writer) {
    fmt.Fprintf(w,"P6\n%d\n%d\n%d\n", t.w, t.h, 255);
    buf := make([]byte, t.w * 3);
    i := 0;
    for y := t.h - 1; y >= 0; y-- {
        o := 0;
        for x := 0; x < t.w; x++ {
            buf[o] = t.buf[i];
            buf[o + 1] = t.buf[i + 1];
            buf[o + 2] = t.buf[i + 2];
            o += 3;
            i += 4;
        }
        w.Write(buf);
    }
}

func formatTGAShort(buf []byte, offset int, value int) {
    buf[offset] = byte(value & 0xff);
    buf[offset+1] = byte((value >> 8) & 0xff);
}

func (t *Texture) WriteTGA(w io.Writer) {
    header := make([]byte, 18);
    header[0] = 0; // ID length
    header[1] = 0; // Color map type
    header[2] = 2; // Image type (2 == uncompressed true-color image)
    header[3] = 0;
    header[4] = 0;
    header[5] = 0;
    header[6] = 0;
    header[7] = 0;
    formatTGAShort(header, 8, 0);
    formatTGAShort(header, 10, 0);
    formatTGAShort(header, 12, t.w);
    formatTGAShort(header, 14, t.h);
    header[16] = 24; // pixel depth
    header[17] = 0;
    
    w.Write(header);
    buf := make([]byte, t.w * 3);
    i := 4 * t.w * (t.h - 1);
    for y := 0; y < t.h; y++ {
        o := 0;
        for x := 0; x < t.w; x++ {
            buf[o] = t.buf[i+2];
            buf[o + 1] = t.buf[i + 1];
            buf[o + 2] = t.buf[i + 0];
            o += 3;
            i += 4;
        }
        i -= 2 * 4 * t.w;
        w.Write(buf);
    }
}

func (t *Texture) SetRgba(x int, y int, r byte, g byte, b byte, a byte) {
    o := 4 * (t.w  * y + x);
    t.buf[o] = r;
    t.buf[o + 1] = g;
    t.buf[o + 2] = b;
    t.buf[o + 3] = a;
}

func f2b(f float) byte {
    scaled := 0.5 + f * 255.0;
    switch {
    case scaled < 0:
        scaled = 0;
    case scaled > 255:
        scaled = 255;
    }
    return byte(scaled);
}

func (t *Texture) SetV(x int, y int, v Vec3) {
    t.SetRgba(x, y, f2b(v.x), f2b(v.y), f2b(v.z), 255);
}

type Rect struct {
    l int;
    t int;
    r int;
    b int;
}

func newRect(l int, t int, r int, b int ) *Rect {
    rect := new(Rect);
    rect.l = l;
    rect.t = t;
    rect.r = r;
    rect.b = b;
    return rect;
}

func (r *Rect) isEmpty() bool {
    return r.l == r.r || r.t == r.b;
}

type Camera struct {
    eye Vec3;
    w int;
    h int;
}

func (c *Camera) rayForPixel(x int, y int) Ray {
    dir := normalize(Vec3{float(x) - float(c.w) * 0.5, float(y) - float(c.h) * 0.5, 
        float(c.w)});
    return Ray{c.eye, dir};
}

type Renderer struct {
    scene *Scene;
    t *Texture;
    cam *Camera;
    jobChan chan Rect;
    quitChan chan bool;
    joinChan chan bool;
}

func (renderer *Renderer) renderRect(tint Vec3, r *Rect) {
    for y := r.t; y < r.b; y++ {
        for x := r.l; x < r.r; x++ {
            ray := renderer.cam.rayForPixel(x, y);
            var g Vec3;
            if false {
                // Draw a rectangle around the bounds of our rendering.
        	    if y == r.t || y == r.b-1 || x == r.l || x == r.r-1 {
        	        g = tint;
        	    } else {
        	        g = renderer.scene.rayTrace(ray);
        	    }
        	} else {
        	    g = renderer.scene.rayTrace(ray);
        	}
            renderer.t.SetV(x, renderer.cam.h - (y + 1), g);
        }
    }
}

func (renderer *Renderer) worker(tint Vec3) {
    jobChan := renderer.jobChan;
    for {
        select {
		case r := <- jobChan:
		    renderer.renderRect(tint, &r);
		case <- renderer.quitChan:
			renderer.joinChan <- true;
			return;
        }
    }
}

func main() {
    level := 8;
    n := 512;
    chunkw := 16;
    chunkh := 16;
    w := n;
    h := n;
    workers := 8;
    t := NewTexture(w, h);
    light := normalize(Vec3{-1.0, -3.0, 2.0});
    sp := createSpherePyramid(level, Vec3{0.0, -1.0, 0.0}, 1.0);
    scene := createScene(light, sp);
    eye := Vec3{0, 0, -4.0};
    camera := Camera{eye, n, n};
    quitChan := make(chan bool);
    joinChan := make(chan bool);
    jobChan := make(chan Rect);
    renderer := Renderer{scene, t, &camera, jobChan, quitChan, joinChan};
    for w := 0; w < workers; w++ {
        tint := Vec3{0.5, float(w) / float(workers), 0.5};
        go renderer.worker(tint);
    }
    for y := 0; y < h; y += chunkh {
        for x := 0; x < w; x += chunkw {
            renderer.jobChan <- Rect{x, y, x + chunkw, y + chunkh};
        }
    }
    for w := 0; w < workers; w++ {
        renderer.quitChan <- true;
    }
    for w := 0; w < workers; w++ {
        <- renderer.joinChan;
    }    
    od, err := os.Open("out.tga", os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0666);
    if err == nil {
        t.WriteTGA(od);
        od.Close();
    }
}
