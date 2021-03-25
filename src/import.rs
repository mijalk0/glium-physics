use crate::material::Material as InternalMaterial;
use crate::mesh::Mesh;
use crate::model::{Model, ModelHandle};
use crate::primitive::Primitive;
use crate::renderer::RendererState;
use crate::skybox::Skybox;
use crate::vertex;
use crate::vertex::Vertex;
use glium::backend::Facade;
use glium::framebuffer::{RenderBuffer, SimpleFrameBuffer};
use glium::texture::{
    Cubemap, MipmapsOption, RawImage2d, SrgbTexture2d, Texture2d, UncompressedFloatFormat,
};
use glium::Program;
use glium::Surface;
use gltf::accessor::{Accessor, DataType};
use gltf::buffer::Source as BufferSource;
use gltf::image::Source as ImageSource;
use gltf::iter::Buffers;
use gltf::material::Material as GltfMaterial;
use gltf::mesh::Semantic;
use gltf::scene::Transform;
use gltf::Gltf;
use image::codecs::hdr::HdrDecoder;
use image::io::Reader;
use image::ImageFormat;
use na::{Isometry3, Matrix3, Quaternion, Translation3, UnitQuaternion, Vector3};
use num::NumCast;
use std::convert::TryInto;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result};
use std::path::Path;

use std::fmt::Debug;

pub const MISSING_TEXTURE_PATH: &str = "assets/missing_texture.png";

// Used to load all gltf buffers into data
pub fn load_buffers<P>(buffers: Buffers, buffer_location: P) -> Vec<Vec<u8>>
where
    P: AsRef<Path> + Debug,
{
    let mut buffer_vec: Vec<Vec<u8>> = Vec::new();
    for buffer in buffers {
        if let BufferSource::Uri(path) = buffer.source() {
            println!("buffer path: {:#?}", buffer_location);
            let path = buffer_location.as_ref().join(path);
            let buffer_binary = fs::read(path).unwrap();
            buffer_vec.push(buffer_binary);
        }
    }
    buffer_vec
}

// Used to load indivudial buffer
fn load_buffer_data<T: NumCast>(
    buffer: &[u8],
    offset: usize,
    count: usize,
    data_type: DataType,
    dimension: usize,
    stride: usize,
) -> Vec<T> {
    let mut data: Vec<T> = Vec::with_capacity(count);

    for i in 0..count {
        for dimension_offset in 0..dimension {
            let start = offset + (i * stride) + (dimension_offset * data_type.size());
            let end = start + data_type.size();
            let bytes = &buffer[start..end];
            //TODO find crate to do this
            //let index: T = T::from_le_bytes(bytes.try_into().unwrap());
            let value: T = match data_type {
                DataType::I8 => {
                    num::cast::<i8, T>(i8::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
                DataType::U8 => {
                    num::cast::<u8, T>(u8::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
                DataType::I16 => {
                    num::cast::<i16, T>(i16::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
                DataType::U16 => {
                    num::cast::<u16, T>(u16::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
                DataType::U32 => {
                    num::cast::<u32, T>(u32::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
                DataType::F32 => {
                    num::cast::<f32, T>(f32::from_le_bytes(bytes.try_into().unwrap())).unwrap()
                }
            };
            data.push(value);
        }
    }

    data
}

fn accessor_as<T: NumCast>(accessor: Accessor, buffers: &[&[u8]]) -> Vec<T> {
    let data_type = accessor.data_type();
    let buffer_view = accessor.view().unwrap();
    let buffer_stride = buffer_view.stride().unwrap_or(accessor.size());
    let buffer = &buffers[buffer_view.buffer().index()];

    load_buffer_data::<T>(
        buffer,
        buffer_view.offset() + accessor.offset(),
        accessor.count(),
        data_type,
        accessor.dimensions().multiplicity(),
        buffer_stride,
    )
}

pub fn load_indices<T: NumCast>(accessor: Accessor, buffers: &[&[u8]]) -> Vec<T> {
    accessor_as::<T>(accessor, buffers)
}

pub fn load_2d_array<T: NumCast + Copy>(accessor: Accessor, buffers: &[&[u8]]) -> Vec<[T; 2]> {
    let data = accessor_as::<T>(accessor, buffers);
    let mut values: Vec<[T; 2]> = Vec::with_capacity(data.len() / 2);
    let chunks = data.chunks(2);

    for chunk in chunks {
        values.push(chunk.try_into().unwrap());
    }

    values
}

pub fn load_3d_array<T: NumCast + Copy>(accessor: Accessor, buffers: &[&[u8]]) -> Vec<[T; 3]> {
    let data = accessor_as::<T>(accessor, buffers);
    let mut values: Vec<[T; 3]> = Vec::with_capacity(data.len() / 3);
    let chunks = data.chunks(3);

    for chunk in chunks {
        values.push(chunk.try_into().unwrap());
    }

    values
}

fn try_load_rawimage_hdr<P>(image_path: P) -> Result<RawImage2d<'static, f32>>
where
    P: AsRef<Path>,
{
    let file = File::open(&image_path)?;
    let reader = BufReader::new(file);
    //    let mut reader = Reader::open(&image_path)?;
    let decoder = HdrDecoder::new(reader).or_else(|_| {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Failed to use hdr bufreader",
        ));
    })?;

    let meta_data = decoder.metadata();
    let width = meta_data.width;
    let height = meta_data.height;
    let dimensions = (width, height);

    let pixels_rgb = decoder
        .read_image_hdr()
        .or_else(|_| return Err(Error::new(ErrorKind::NotFound, "Failed to read hdr image")))?;

    let mut pixels_data = Vec::with_capacity(pixels_rgb.len() * 3);

    println!("Starting long hdr load, copying pixels");
    for i in 0..pixels_rgb.len() {
        pixels_data.push(pixels_rgb[i][0]);
        pixels_data.push(pixels_rgb[i][1]);
        pixels_data.push(pixels_rgb[i][2]);
    }
    println!("Finished long hdr loading");

    println!("{:?}", dimensions);
    Ok(RawImage2d::from_raw_rgb_reversed(&pixels_data, dimensions))
}

fn load_rawimage_hdr<P>(image_path: P) -> RawImage2d<'static, f32>
where
    P: AsRef<Path>,
{
    match try_load_rawimage_hdr(image_path) {
        Ok(image) => image,
        Err(_) => try_load_rawimage_hdr(MISSING_TEXTURE_PATH).unwrap(),
    }
}

fn load_hdr_texture<F: ?Sized, P>(facade: &F, image_path: P) -> Texture2d
where
    F: Facade,
    P: AsRef<Path>,
{
    let image = load_rawimage_hdr(image_path);
    Texture2d::with_format(
        facade,
        image,
        UncompressedFloatFormat::F32F32F32,
        MipmapsOption::NoMipmap,
    )
    .unwrap()
}

fn try_load_rawimage_rgba<P>(image_path: P) -> Result<RawImage2d<'static, u8>>
where
    P: AsRef<Path>,
{
    let mut reader = Reader::open(&image_path)?;
    if let Some(os_extension) = image_path.as_ref().extension() {
        if let Some(extension) = os_extension.to_str() {
            match extension {
                ".png" => {
                    reader.set_format(ImageFormat::Png);
                }
                ".tga" => {
                    reader.set_format(ImageFormat::Tga);
                }
                ".tiff" => {
                    reader.set_format(ImageFormat::Tiff);
                }
                ".jpeg" => {
                    reader.set_format(ImageFormat::Jpeg);
                }
                ".bmp" => {
                    reader.set_format(ImageFormat::Bmp);
                }
                _ => {
                    reader = reader.with_guessed_format()?;
                }
            }
        } else {
            reader = reader.with_guessed_format()?
        }
    } else {
        reader = reader.with_guessed_format()?
    };

    let image = reader
        .decode()
        .or_else(|_| {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Failed to load raw rgba image",
            ));
        })?
        .into_rgba8();
    let dimensions = image.dimensions();

    println!("{:?}", dimensions);
    Ok(RawImage2d::from_raw_rgba(image.into_raw(), dimensions))
}

fn load_rawimage_rgba<P>(image_path: P) -> RawImage2d<'static, u8>
where
    P: AsRef<Path>,
{
    match try_load_rawimage_rgba(image_path) {
        Ok(image) => image,
        Err(_) => try_load_rawimage_rgba(MISSING_TEXTURE_PATH).unwrap(),
    }
}

fn load_srgb_texture<F: ?Sized, P>(facade: &F, image_path: P) -> SrgbTexture2d
where
    F: Facade,
    P: AsRef<Path>,
{
    let image = load_rawimage_rgba(image_path);
    SrgbTexture2d::new(facade, image).unwrap()
}

fn load_missing_srgb_texture<F: ?Sized>(facade: &F) -> SrgbTexture2d
where
    F: Facade,
{
    load_srgb_texture(facade, MISSING_TEXTURE_PATH)
}

fn load_rgba_texture<F: ?Sized, P>(facade: &F, image_path: P) -> Texture2d
where
    F: Facade,
    P: AsRef<Path>,
{
    let image = load_rawimage_rgba(image_path);
    Texture2d::new(facade, image).unwrap()
}

fn load_missing_rgba_texture<F: ?Sized>(facade: &F) -> Texture2d
where
    F: Facade,
{
    load_rgba_texture(facade, MISSING_TEXTURE_PATH)
}

fn load_missing_material<F: ?Sized>(facade: &F) -> InternalMaterial
where
    F: Facade,
{
    let diffuse_map = load_missing_srgb_texture(facade);
    let orm_map = load_missing_rgba_texture(facade);
    let normal_map = load_missing_rgba_texture(facade);
    InternalMaterial::new(diffuse_map, orm_map, normal_map)
}

pub fn load_material<F: ?Sized, P>(
    facade: &F,
    material: GltfMaterial,
    material_location: P,
) -> InternalMaterial
where
    F: Facade,
    P: AsRef<Path>,
{
    let diffuse_map_source = match material.pbr_metallic_roughness().base_color_texture() {
        Some(texture) => texture,
        None => return load_missing_material(facade),
    }
    .texture()
    .source()
    .source();

    let diffuse_map = if let ImageSource::Uri { uri: path, .. } = diffuse_map_source {
        load_srgb_texture(facade, material_location.as_ref().join(path))
    } else {
        return load_missing_material(facade);
    };

    let orm_map_source = match material
        .pbr_metallic_roughness()
        .metallic_roughness_texture()
    {
        Some(texture) => texture,
        None => return load_missing_material(facade),
    }
    .texture()
    .source()
    .source();

    let orm_map = if let ImageSource::Uri { uri: path, .. } = orm_map_source {
        load_rgba_texture(facade, material_location.as_ref().join(path))
    } else {
        return load_missing_material(facade);
    };

    let normal_map_source = match material.normal_texture() {
        Some(texture) => texture,
        None => return load_missing_material(facade),
    }
    .texture()
    .source()
    .source();

    let normal_map = if let ImageSource::Uri { uri: path, .. } = normal_map_source {
        load_rgba_texture(facade, material_location.as_ref().join(path))
    } else {
        return load_missing_material(facade);
    };

    InternalMaterial::new(diffuse_map, orm_map, normal_map)
}

pub fn model_from_gltf<F: ?Sized, P>(
    facade: &F,
    rs: &mut RendererState,
    path: P,
) -> Result<ModelHandle>
where
    F: Facade,
    P: AsRef<Path> + Debug,
{
    let gltf_path = path
        .as_ref()
        .join(path.as_ref().file_stem().unwrap().to_str().unwrap())
        .with_extension("gltf");

    println!("{:?}", gltf_path);

    let gltf = Gltf::open(gltf_path).expect("Failed to load AKM gltf file.");

    let buffers: Vec<Vec<u8>> = load_buffers(gltf.buffers(), &path);
    let buffers: Vec<&[u8]> = buffers.iter().map(|buffer| buffer.as_slice()).collect();
    let buffer_slices = buffers.as_slice();

    let mut meshes: Vec<Mesh> = Vec::new();
    let mut materials: Vec<InternalMaterial> = Vec::new();

    for material in gltf.materials() {
        materials.push(load_material(facade, material, &path));
    }

    let scene = gltf.default_scene().unwrap();
    //        let scene = gltf.default_scene().unwrap_or_else(|| {
    //            Err(Error::new(ErrorKind::NotFound, "Failed to unwrap scene from gltf."))
    //        });

    for node in scene.nodes() {
        let mesh = node.mesh().expect("Failed to unwrap mesh from node.");
        let transformation_matrix = node.transform();
        let (mesh_isometry, mesh_scaling) = match transformation_matrix {
            Transform::Matrix { matrix } => {
                let position = Translation3::new(matrix[3][0], matrix[3][1], matrix[3][2]);
                //TODO check that the rotation matrix created is done properly
                let rotation = UnitQuaternion::from_matrix(&Matrix3::new(
                    matrix[0][0],
                    matrix[0][1],
                    matrix[0][2],
                    matrix[1][0],
                    matrix[1][1],
                    matrix[1][2],
                    matrix[2][0],
                    matrix[2][1],
                    matrix[2][2],
                ));

                let isometry = Isometry3::from_parts(position, rotation);
                let scaling = Vector3::from_element(1.0);
                (isometry, scaling)
            }
            Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                let translation_vec = Vector3::new(translation[0], translation[1], translation[2]);
                let rotation_vec = Vector3::new(rotation[0], rotation[1], rotation[2]);
                let scale_vec = Vector3::new(scale[0], scale[1], scale[2]);

                let quaternion = UnitQuaternion::from_quaternion(Quaternion::from_parts(
                    rotation[3],
                    rotation_vec,
                ));

                let axis_angle = match quaternion.axis() {
                    Some(axis_angle) => axis_angle.into_inner(),
                    None => Vector3::new(0.0, 0.0, 0.0),
                };

                let isometry = Isometry3::new(translation_vec, axis_angle);
                let scaling = scale_vec;

                (isometry, scaling)
            }
        };
        let mut primitives: Vec<Primitive> = Vec::new();

        for primitive in mesh.primitives() {
            let indices_accessor = primitive
                .indices()
                .expect("Failed to unwrap indices from primitive.");

            let indices: Vec<u32> = load_indices::<u32>(indices_accessor, buffer_slices);

            let mut vertices: Vec<Vertex> = Vec::new();

            let mut vertex_positions: Vec<[f32; 3]> = Vec::with_capacity(indices.len());
            let mut vertex_texture_coords: Vec<[f32; 2]> = Vec::with_capacity(indices.len());
            let mut vertex_normals: Vec<[f32; 3]> = Vec::with_capacity(indices.len());
            let mut vertex_tangents: Vec<[f32; 3]> = Vec::with_capacity(indices.len());

            for attribute in primitive.attributes() {
                let vertex_attribute = attribute.0;
                let accessor = attribute.1;

                match vertex_attribute {
                    Semantic::Positions => {
                        vertex_positions = load_3d_array::<f32>(accessor, &buffer_slices);
                    }
                    Semantic::Normals => {
                        vertex_normals = load_3d_array::<f32>(accessor, &buffer_slices);
                    }
                    Semantic::TexCoords(0) => {
                        vertex_texture_coords = load_2d_array::<f32>(accessor, &buffer_slices);
                        vertex_texture_coords = vertex_texture_coords
                            .into_iter()
                            .map(|mut texture_coord| {
                                let u = texture_coord[0];
                                let v = texture_coord[1];

                                if u > 1f32 {
                                    texture_coord[0] = u - u.floor();
                                } else if u < 0f32 {
                                    texture_coord[0] = 1f32 - (u.abs() - u.abs().floor());
                                }

                                if v > 1f32 {
                                    texture_coord[1] = v - v.floor();
                                } else if v < 0f32 {
                                    texture_coord[1] = 1f32 - (v.abs() - v.abs().floor());
                                }
                                texture_coord
                            })
                            .collect();
                    }
                    _ => (),
                }
            }

            // Calculating tangents
            println!("{:?}", indices.len());
            for i in 0..indices.len() / 3 {
                let index1 = indices[i * 3] as usize;
                let index2 = indices[i * 3 + 1] as usize;
                let index3 = indices[i * 3 + 2] as usize;

                let position1 = vertex_positions[index1];
                let position2 = vertex_positions[index2];
                let position3 = vertex_positions[index3];

                let texture_coord1 = vertex_texture_coords[index1];
                let texture_coord2 = vertex_texture_coords[index2];
                let texture_coord3 = vertex_texture_coords[index3];

                let tangent = vertex::calculate_tangent(
                    position1,
                    position2,
                    position3,
                    texture_coord1,
                    texture_coord2,
                    texture_coord3,
                );
                vertex_tangents.push(tangent);
            }

            for i in 0..vertex_positions.len() {
                vertices.push(Vertex::new(
                    vertex_positions[i],
                    vertex_texture_coords[i],
                    vertex_normals[i],
                    vertex_tangents[i / 3],
                ));
            }

            let material_index = primitive.material().index().unwrap();
            let primitive = Primitive::new(facade, vertices, indices, material_index);
            primitives.push(primitive);
        }
        meshes.push(Mesh::new(primitives, mesh_isometry, mesh_scaling));
    }
    let model = Model::new(meshes, materials);
    let model_handle = rs.push_model(model);
    Ok(model_handle)
    //    Err(Error::new(ErrorKind::NotFound, "Failed to load gltf file."))
}

//TODO Refactor this to work with non IBL textures somehow. Maybe make another skybox struct without
//ibl support.

//pub fn load_skybox<F: ?Sized, P>(facade: &F, path: P) -> Skybox
//where
//    F: Facade,
//    P: AsRef<Path>,
//{
//    let extension = if let Some(os_extension) = path.as_ref().extension() {
//        if let Some(os_extension) = os_extension.to_str() {
//            os_extension
//        } else {
//            "png"
//        }
//    } else {
//        "png"
//    };
//
//    let textures = [
//        load_rgba_texture(facade, path.as_ref().join("posx").with_extension(extension)),
//        load_rgba_texture(facade, path.as_ref().join("negx").with_extension(extension)),
//        load_rgba_texture(facade, path.as_ref().join("posy").with_extension(extension)),
//        load_rgba_texture(facade, path.as_ref().join("negy").with_extension(extension)),
//        load_rgba_texture(facade, path.as_ref().join("posz").with_extension(extension)),
//        load_rgba_texture(facade, path.as_ref().join("negz").with_extension(extension)),
//    ];
//    let cubemap = Cubemap::empty(facade, textures[0].width()).unwrap();
//
//    Skybox::new(facade, cubemap, textures)
//}

pub fn load_skybox_from_hdr<F: ?Sized, P>(
    facade: &F,
    path: P,
    hdr_program: Program,
    irradiance_program: Program,
    prefiltered_program: Program,
    brdf_integration_program: Program,
) -> Skybox
where
    F: Facade,
    P: AsRef<Path>,
{
    let hdr_texture = load_hdr_texture(facade, path.as_ref());

    let width = 2048;
    let height = 2048;

    let rect = glium::Rect {
        left: 0,
        bottom: 0,
        width,
        height,
    };

    // Mipmaps on may or may not cause undefined behaviour, need to check
    let float_format = glium::texture::UncompressedFloatFormat::F32F32F32;
    let mipmaps_option = glium::texture::MipmapsOption::EmptyMipmaps;
    let cubemap = Cubemap::empty_with_format(facade, float_format, mipmaps_option, width).unwrap();

    let vbo = Skybox::make_vbo(facade);
    let ibo = Skybox::make_ibo(facade);

    // 0 = posx
    // 1 = negx
    // 2 = posy
    // 3 = negy
    // 4 = posz
    // 5 = negz

    let projection_matrix = Skybox::cubemap_projection_matrix();
    let view_matrices = Skybox::cubemap_view_matrices();

    let draw_parameters = glium::DrawParameters {
        viewport: Some(rect),
        ..Default::default()
    };

    let cubemap_framebuffers = Skybox::cubemap_framebuffers(facade, &cubemap);

    // Drawing to the frame buffers for i in 0..6
    for i in 0..view_matrices.len() {
        let renderbuffer = RenderBuffer::new(facade, float_format, width, height).unwrap();
        let mut framebuffer = SimpleFrameBuffer::new(facade, &renderbuffer).unwrap();

        let uniforms = uniform!(
            hdr_texture: &hdr_texture,
            projection_matrix: projection_matrix,
            view_matrix: view_matrices[i]
        );

        // Copying the final frame buffers into the cube map texures
        framebuffer
            .draw(&vbo, &ibo, &hdr_program, &uniforms, &draw_parameters)
            .unwrap();
        framebuffer.fill(
            &cubemap_framebuffers[i],
            glium::uniforms::MagnifySamplerFilter::Linear,
        );
    }

    unsafe {
        cubemap.generate_mipmaps();
    }

    Skybox::from_cubemap(
        facade,
        cubemap,
        irradiance_program,
        prefiltered_program,
        7,
        brdf_integration_program,
        (512, 512),
    )
}

pub fn load_program<F>(facade: &F, vertex_src: &str, fragment_src: &str) -> Program
where
    F: Facade,
{
    let program = program!(facade, 330 => {
    vertex: vertex_src,
    fragment: fragment_src,
    outputs_srgb: true
    })
    .unwrap();
    program
}

//TODO make function to load attributes

//Both functions should use a generic T and make use of the load_buffer function
//Likely will not have to use the Target enum because glium already handles its purpose
