# glium-physics
Work in progress rendering + physics engine. Uses standard PBR lighting techniques. Currently supports albedo/color, normal, roughness, metal, and ambient occlusion maps. Supports physics via the rapier rust crate.

<img width="1149" alt="image" src="https://user-images.githubusercontent.com/25313161/112530796-400db280-8d7d-11eb-8e4a-353697add8e5.png">

<img width="1160" alt="image" src="https://user-images.githubusercontent.com/25313161/112530834-48fe8400-8d7d-11eb-94e7-775411644365.png">

![mov-to-gif-1-3](https://user-images.githubusercontent.com/25313161/112530884-5287ec00-8d7d-11eb-995d-597420fce6a5.gif)

Showcasing PBR textures and image based lighting. There are no external lights in this scene, the model is lit up exclusively by the skybox.

<img width="1131" alt="image" src="https://user-images.githubusercontent.com/25313161/112532753-92e86980-8d7f-11eb-8bbb-95c2297429dc.png">

Different `.hdr` skybox.

![glium-physics showcas](https://user-images.githubusercontent.com/25313161/112530981-6c293380-8d7d-11eb-9dfc-518f09886463.gif)

Sample physics demo. The purple texture is a phyiscs collision object made up of triangles (trimesh) and the model bouncing is using a box for the collision detection. Both have high restitution in this demo.
