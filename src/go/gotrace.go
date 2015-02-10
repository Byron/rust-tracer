// Adapted from http://www.ffconsultancy.com/languages/ray_tracer/comparison.html
// Original Author: Jack Palevich
// Performance Improvements: Sebastian Thiel

package main

import fmt "fmt"
import io "io"
import os "os"
import math "math"

var infinity float32 = float32(math.Inf(1))
var delta float32 = float32(math.Sqrt(1.19209E-07)) // sqrt(float_epsilon)

func sqrtf(a float32) float32 {
	return float32(math.Sqrt(float64(a)))
}

type Vec3 struct {
	x, y, z float32
}

func (v *Vec3) add(b *Vec3) *Vec3 {
	v.x += b.x
	v.y += b.y
	v.z += b.z
	return v
}

func (v *Vec3) sub(b *Vec3) *Vec3 {
	v.x -= b.x
	v.y -= b.y
	v.z -= b.z
	return v
}

func (v *Vec3) mulf(b float32) *Vec3 {
	v.x *= b
	v.y *= b
	v.z *= b
	return v
}

func (v *Vec3) normalize() *Vec3 {
	return v.mulf(1.0 / sqrtf(v.dot(v)))
}

func (v Vec3) normalized() Vec3 {
	return *v.mulf(1.0 / sqrtf(v.dot(&v)))
}

func (v *Vec3) dot(b *Vec3) float32 {
	return v.x*b.x + v.y*b.y + v.z*b.z
}

func vec3add(a Vec3, b Vec3) Vec3 {
	a.x += b.x
	a.y += b.y
	a.z += b.z
	return a
}

func vec3sub(a Vec3, b Vec3) Vec3 {
	a.x -= b.x
	a.y -= b.y
	a.z -= b.z
	return a
}

func vec3mulf(a Vec3, b float32) Vec3 {
	a.x *= b
	a.y *= b
	a.z *= b
	return a
}

func vec3dot(a Vec3, b Vec3) float32 {
	return a.x*b.x + a.y*b.y + a.z*b.z
}

func normalize(a Vec3) Vec3 {
	return vec3mulf(a, 1.0/sqrtf(vec3dot(a, a)))
}

var backgroundColor Vec3 = Vec3{0.1, 0.1, 0.1}
var diffuseSphereColor Vec3 = Vec3{0.0, 0.7, 0.0}
var ambientSphereColor Vec3 = Vec3{0.2, 0.3, 0.2}

type Sphere struct {
	center Vec3
	radius float32
}

type Hit struct {
	distance float32
	pos      Vec3
}

var hitinfinity Hit = Hit{infinity, Vec3{0, 0, 0}}

type Ray struct {
	orig, dir Vec3
}

type Geometry interface {
	Intersect(h *Hit, r *Ray)
	Print() // Temporary until fmt handles interfaces.
}

func (s *Sphere) RaySphere(r *Ray) float32 {
	v := vec3sub(s.center, r.orig)
	b := vec3dot(v, r.dir)
	disc := b*b - vec3dot(v, v) + s.radius*s.radius
	if disc < 0.0 {
		return infinity
	}
	d := sqrtf(disc)
	t2 := b + d
	if t2 < 0.0 {
		return infinity
	}
	t1 := b - d
	if t1 > 0.0 {
		return t1
	}
	return t2
}

func (s *Sphere) Intersect(h *Hit, r *Ray) {
	lambda := s.RaySphere(r)
	if lambda >= h.distance {
		return
	}
	h.distance = lambda
	h.pos = normalize(vec3add(r.orig, vec3sub(vec3mulf(r.dir, lambda), s.center)))
}

func (s *Sphere) Print() {
	fmt.Println("Sphere:", *s)
}

type Group struct {
	bound    Sphere
	children []Geometry
}

func (g *Group) Print() {
	fmt.Print("Group:")
	g.bound.Print()
	for i := 0; i < len(g.children); i++ {
		fmt.Print("  ")
		g.children[i].Print()
	}
}

func (g *Group) Intersect(h *Hit, r *Ray) {
	l := g.bound.RaySphere(r)
	if l >= h.distance {
		return
	}
	for _, c := range g.children {
		c.Intersect(h, r)
	}
}

func NewGroup(bound Sphere, children []Geometry) *Group {
	g := new(Group)
	g.bound = bound
	g.children = children
	return g
}

type Scene struct {
	light Vec3
	g     Geometry
}

func createScene(light Vec3, g Geometry) *Scene {
	scene := new(Scene)
	scene.light = light
	scene.g = g
	return scene
}

func (s *Scene) rayTrace(r *Ray) Vec3 {
	var hit Hit = hitinfinity
	s.g.Intersect(&hit, r)
	if hit.distance == infinity {
		return backgroundColor
	}
	g := vec3dot(hit.pos, s.light)
	if g >= 0.0 {
		// The hit intersection is in shadow
		return ambientSphereColor
	}
	p := vec3add(r.orig, vec3add(vec3mulf(r.dir, hit.distance), vec3mulf(hit.pos, delta)))
	hit.distance = infinity
	s.g.Intersect(&hit, &Ray{p, vec3mulf(s.light, -1.0)})
	if hit.distance < infinity {
		// There`s an object between us and the light.
		return ambientSphereColor
	}
	litColor := vec3mulf(diffuseSphereColor, -g)
	totalColor := vec3add(ambientSphereColor, litColor)
	return totalColor
}

func createSpherePyramid(level int, c Vec3, r float32) Geometry {
	s := new(Sphere)
	s.center = c
	s.radius = r
	if level == 1 {
		return s
	}
	children := make([]Geometry, 5)
	i := 0
	children[i] = s
	i++
	rn := 3.0 * r / sqrtf(12.0)
	for dz := -1; dz <= 1; dz += 2 {
		for dx := -1; dx <= 1; dx += 2 {
			newc := vec3add(c, vec3mulf(Vec3{float32(dx), 1.0, float32(dz)}, rn))
			children[i] = createSpherePyramid(level-1, newc, r*0.5)
			i++
		}
	}
	return NewGroup(Sphere{c, 3 * r}, children)
}

type Texture struct {
	w, h int
	buf  []byte
}

func NewTexture(w int, h int) *Texture {
	t := new(Texture)
	t.w = w
	t.h = h
	t.buf = make([]byte, w*h*4)
	return t
}

func formatTGAShort(buf []byte, offset int, value int) {
	buf[offset] = byte(value & 0xff)
	buf[offset+1] = byte((value >> 8) & 0xff)
}

func (t *Texture) WriteTGA(w io.Writer) {
	header := make([]byte, 18)
	header[0] = 0 // ID length
	header[1] = 0 // Color map type
	header[2] = 2 // Image type (2 == uncompressed true-color image)
	header[3] = 0
	header[4] = 0
	header[5] = 0
	header[6] = 0
	header[7] = 0
	formatTGAShort(header, 8, 0)
	formatTGAShort(header, 10, 0)
	formatTGAShort(header, 12, t.w)
	formatTGAShort(header, 14, t.h)
	header[16] = 24 // pixel depth
	header[17] = 0

	w.Write(header)
	buf := make([]byte, t.w*3)
	i := 4 * t.w * (t.h - 1)
	for y := 0; y < t.h; y++ {
		o := 0
		for x := 0; x < t.w; x++ {
			buf[o] = t.buf[i+2]
			buf[o+1] = t.buf[i+1]
			buf[o+2] = t.buf[i+0]
			o += 3
			i += 4
		}
		i -= 2 * 4 * t.w
		w.Write(buf)
	}
}

func (t *Texture) SetRgba(x int, y int, r byte, g byte, b byte, a byte) {
	o := 4 * (t.w*y + x)
	t.buf[o] = r
	t.buf[o+1] = g
	t.buf[o+2] = b
	t.buf[o+3] = a
}

func f2b(f float32) byte {
	scaled := 0.5 + f*255.0
	switch {
	case scaled < 0:
		scaled = 0
	case scaled > 255:
		scaled = 255
	}
	return byte(scaled)
}

func (t *Texture) SetV(x int, y int, v Vec3) {
	t.SetRgba(x, y, f2b(v.x), f2b(v.y), f2b(v.z), 255)
}

type Rect struct {
	l int
	t int
	r int
	b int
}

func newRect(l, t, r, b int) *Rect {
	rect := new(Rect)
	rect.l = l
	rect.t = t
	rect.r = r
	rect.b = b
	return rect
}

func (r *Rect) isEmpty() bool {
	return r.l == r.r || r.t == r.b
}

type Camera struct {
	eye Vec3
	w   int
	h   int
}

func (c *Camera) setRayDirForPixel(r *Ray, x, y float32) {
	r.dir.x = x - float32(c.w)*0.5
	r.dir.y = y - float32(c.h)*0.5
	r.dir.z = float32(c.w)
	r.dir.normalize()
}

type Renderer struct {
	scene      *Scene
	t          *Texture
	cam        *Camera
	ss         int // oversampling
	xres, yres int // image resolution
	jobChan    chan Rect
	quitChan   chan bool
	joinChan   chan bool
}

func (ren *Renderer) renderRect(tint Vec3, r *Rect) {
	ray := Ray{orig: ren.cam.eye}

	for y := r.t; y < r.b; y++ {
		for x := r.l; x < r.r; x++ {
			var g Vec3
			for ssx := 0; ssx < ren.ss; ssx++ {
				for ssy := 0; ssy < ren.ss; ssy++ {
					var xres float32 = float32(x) + float32(ssx)/float32(ren.ss)
					var yres float32 = float32(y) + float32(ssy)/float32(ren.ss)

					ren.cam.setRayDirForPixel(&ray, xres, yres)
					g = vec3add(g, ren.scene.rayTrace(&ray))
				} // END for each y subsample
			} // END for each x subsample

			ren.t.SetV(x, ren.cam.h-(y+1), vec3mulf(g, 1.0/float32(ren.ss*ren.ss)))

		} // END for each x pixel
	} // END for each y pixel
}

func (renderer *Renderer) worker(tint Vec3) {
	jobChan := renderer.jobChan
	for {
		select {
		case r := <-jobChan:
			renderer.renderRect(tint, &r)
		case <-renderer.quitChan:
			renderer.joinChan <- true
			return
		}
	}
}

func main() {
	level := 8
	n := 1024
	chunkw := 16
	chunkh := 16
	w := n
	h := n
	workers := 8
	ss := 4 // oversampling - use 4 to get 16 samples
	t := NewTexture(w, h)
	light := normalize(Vec3{-1.0, -3.0, 2.0})
	sp := createSpherePyramid(level, Vec3{0.0, -1.0, 0.0}, 1.0)
	scene := createScene(light, sp)
	eye := Vec3{0, 0, -4.0}
	camera := Camera{eye, n, n}
	quitChan := make(chan bool)
	joinChan := make(chan bool)
	jobChan := make(chan Rect)
	renderer := Renderer{scene, t, &camera, ss, w, h, jobChan, quitChan, joinChan}
	for w := 0; w < workers; w++ {
		tint := Vec3{0.5, float32(w) / float32(workers), 0.5}
		go renderer.worker(tint)
	}
	for y := 0; y < h; y += chunkh {
		for x := 0; x < w; x += chunkw {
			renderer.jobChan <- Rect{x, y, x + chunkw, y + chunkh}
		}
	}
	for w := 0; w < workers; w++ {
		renderer.quitChan <- true
	}
	for w := 0; w < workers; w++ {
		<-renderer.joinChan
	}
	od, err := os.OpenFile("out.tga", os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0666)
	if err == nil {
		t.WriteTGA(od)
		od.Close()
	}
}
