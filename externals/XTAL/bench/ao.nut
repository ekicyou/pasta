
IMAGEWIDTH <- 32;
IMAGEHEIGHT <- 32;
NSUBSAMPLES <- 2;
NAOSAMPLES <- 8;

function random(){
	return rand().tofloat()/RAND_MAX;
}

class Vec{
	x = 0.0;
	y = 0.0;
	z = 0.0;
	
	constructor(ax, ay, az){
		x = ax;
		y = ay;
		z = az;
	}
	
	function _add(a){
		return Vec(x+a.x, y+a.y, z+a.z);
	}
	
	function _sub(a){
		return Vec(x-a.x, y-a.y, z-a.z);
	}
	
	function length(){
		return sqrt(x * x + y * y + z * z);
	}
	
	function normalize(){
		local len = length();
		if(abs(len) > 1.0e-17) {
		   return Vec(x/len, y/len, z/len);
		}
		return this;	
	}
	
	function cross(a){
		return Vec(y*a.z-z*a.y, z*a.x-x*a.z, x*a.y-y*a.x);
	}

	function dot(a){
		return x * a.x + y * a.y + z * a.z;
	}
}

class Sphere{
	center = null;
	radius = 0.0;
	
	constructor(acenter, aradius){
		center = acenter;
		radius = aradius;
	}
	
	function intersect(ray, isect){
		local rs =  ray.org - center;
		local B = rs.dot(ray.dir);
		local C = rs.dot(rs) - (radius * radius);
		local D = B * B - C;

		if(D > 0.0){
			local t = -B - sqrt(D);

			if((t > 0.0) && (t < isect.t)){
				isect.t = t;
				isect.hit = true;
				isect.p = Vec(ray.org.x + ray.dir.x * t, ray.org.y + ray.dir.y * t, ray.org.z + ray.dir.z * t);
				local n = isect.p - center;
				isect.n = n.normalize();
			}
		}
	}	
}

class Plane{
	p = null;
	n = null;

	constructor(ap, an){
		p = ap;
		n = an;
	}

	function intersect(ray, isect){
		local d = -p.dot(n);
		local v = ray.dir.dot(n);

		if(fabs(v) < 1.0e-17) return;

		local t = -(ray.org.dot(n) + d) / v;

		if((t > 0.0) && (t < isect.t)){
			isect.hit = true;
			isect.t = t;
			isect.n = n;
			isect.p = Vec(ray.org.x + t * ray.dir.x, ray.org.y + t * ray.dir.y, ray.org.z + t * ray.dir.z);
		}
	}
}

class Ray{
	org = null;
	dir = null;

	constructor(aorg, adir){
		org = aorg;
		dir = adir;
	}
}

class Isect{
	t = 0.0;
	hit = false;
	p = null;
	n = null;
	
	constructor(){
    	t = 1000000.0;
    	hit = false;
    	p = Vec(0.0, 0.0, 0.0);
    	n = Vec(0.0, 0.0, 0.0);	
	}
}

function clamp(f){
	local i = f * 255.5;
	if(i > 255.0) i = 255.0;
	if(i < 0.0) i = 0.0;
	return (i+0.5).tointeger(); //round(i);
}

function orthoBasis(basis, n){
	basis[2] = Vec(n.x, n.y, n.z);
	basis[1] = Vec(0.0, 0.0, 0.0);

	if((n.x < 0.6) && (n.x > -0.6)){
		basis[1].x = 1.0;
	} 
	else if((n.y < 0.6) && (n.y > -0.6)){
		basis[1].y = 1.0;
	} 
	else if((n.z < 0.6) && (n.z > -0.6)){
		basis[1].z = 1.0;
	} 
	else {
		basis[1].x = 1.0;
	}

	basis[0] = basis[1].cross(basis[2]);
	basis[0] = basis[0].normalize();

	basis[1] = basis[2].cross(basis[0]);
	basis[1] = basis[1].normalize();
}

spheres <- null;
plane <- null;

function init_scene(){
	spheres = array(3);
	spheres[0] = Sphere(Vec(-2.0, 0.0, -3.5), 0.5);
	spheres[1] = Sphere(Vec(-0.5, 0.0, -3.0), 0.5);
	spheres[2] = Sphere(Vec(1.0, 0.0, -2.2), 0.5);
	plane = Plane(Vec(0.0, -0.5, 0.0), Vec(0.0, 1.0, 0.0));
}

function ambient_occlusion(isect){
	local basis = array(3);
	orthoBasis(basis,  isect.n);

	local ntheta = NAOSAMPLES;
	local nphi = NAOSAMPLES;
	local eps = 0.0001;
	local occlusion = 0.0;

	local p = Vec(isect.p.x + eps * isect.n.x, isect.p.y + eps * isect.n.y, isect.p.z + eps * isect.n.z);

	for(local j = 0; j < nphi; j++) {
		for(local i = 0; i < ntheta; i++) {
			local r = random();
			local phi = 2.0 * PI * random();

			local x = cos(phi) * sqrt(1.0 - r);
			local y = sin(phi) * sqrt(1.0 - r);
			local z = sqrt(r);

			local rx = x * basis[0].x + y * basis[1].x + z * basis[2].x;
			local ry = x * basis[0].y + y * basis[1].y + z * basis[2].y;
			local rz = x * basis[0].z + y * basis[1].z + z * basis[2].z;

			local raydir = Vec(rx, ry, rz);
			local ray = Ray(p, raydir);

			local occIsect = Isect();
			spheres[0].intersect(ray, occIsect);
			spheres[1].intersect(ray, occIsect);
			spheres[2].intersect(ray, occIsect);
			plane.intersect(ray, occIsect);

			if(occIsect.hit){
				occlusion += 1.0;
			}
		}
	}

	occlusion = (ntheta * nphi - occlusion) / (ntheta * nphi);
	return Vec(occlusion, occlusion, occlusion);
}

function render(w, h, nsubsamples){
	local cnt = 0;

	for(local y = 0; y < h; y++) {
		for(local x = 0; x < w; x++) {
			local rad = Vec(0.0, 0.0, 0.0);

			for(local v = 0; v < nsubsamples; v++) {
				for(local u = 0; u < nsubsamples; u++) {

					cnt++;
					local px = (x + (u / nsubsamples) - (w / 2.0))/(w / 2.0);
					local py = -(y + (v / nsubsamples) - (h / 2.0))/(h / 2.0);

					local eye = Vec(px, py, -1.0).normalize();

					local ray = Ray(Vec(0.0, 0.0, 0.0), eye);

					local isect = Isect();
					spheres[0].intersect(ray, isect);
					spheres[1].intersect(ray, isect);
					spheres[2].intersect(ray, isect);
					plane.intersect(ray, isect);

					if(isect.hit){
					   	local col = ambient_occlusion(isect);
						rad.x += col.x;
						rad.y += col.y;
						rad.z += col.z;
					}
				}
			}

			local r = rad.x / (nsubsamples * nsubsamples);
			local g = rad.y / (nsubsamples * nsubsamples);
			local b = rad.z / (nsubsamples * nsubsamples);
			
			//print(format("%c", clamp(r)));
			//print(format("%c", clamp(g)));
			//print(format("%c", clamp(b)));
			stdout.writen(clamp(r), 'b');
			stdout.writen(clamp(g), 'b');
			stdout.writen(clamp(b), 'b');
		}
	}
}

init_scene();

print("P6\n");
print(format("%d %d\n", IMAGEWIDTH, IMAGEHEIGHT));
print("255\n");
render(IMAGEWIDTH, IMAGEHEIGHT, NSUBSAMPLES);


