# glium-physics
Work in progress rendering + physics engine. Uses standard PBR lighting techniques. Currently supports albedo/color, normal, roughness, metal, and ambient occlusion maps. Supports physics via the rapier rust crate.

![glium-physics showcase](https://user-images.githubusercontent.com/25313161/112500532-d9799c00-8d5e-11eb-97fb-ff8678b8044a.mov)

Showcasing PBR textures and image based lighting. There are no external lights in this scene, the model is lit up exclusively by the skybox.

![glium-physics showcase](https://user-images.githubusercontent.com/25313161/112499662-142f0480-8d5e-11eb-8cf2-bab2c18701f0.mov)

Sample physics demo. The purple texture is a phyiscs collision object made up of triangles (trimesh) and the model bouncing is using a box for the collision detection.
