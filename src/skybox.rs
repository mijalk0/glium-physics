use crate::vertex::{QuadVertex, SkyboxVertex};
use glium::backend::Facade;
use glium::framebuffer::{RenderBuffer, SimpleFrameBuffer};
use glium::texture::{CubeLayer, Cubemap, Texture2d};
use glium::uniforms::{MagnifySamplerFilter, Sampler};
use glium::BlitTarget;
use glium::Program;
use glium::Surface;
use glium::{IndexBuffer, VertexBuffer};
use nalgebra::{Isometry3, Perspective3, Point3, Vector3};

pub struct Skybox {
    pub cubemap: Cubemap,
    pub irradiance_map: Cubemap,
    pub prefiltered_map: Cubemap,
    pub brdf_integration: Texture2d,
    pub vbo: VertexBuffer<SkyboxVertex>,
    pub ibo: IndexBuffer<u32>,
}

impl Skybox {
    pub fn new<F: ?Sized>(
        facade: &F,
        cubemap: Cubemap,
        irradiance_map: Cubemap,
        prefiltered_map: Cubemap,
        brdf_integration: Texture2d,
    ) -> Self
    where
        F: Facade,
    {
        let vbo = Self::make_vbo(facade);
        let ibo = Self::make_ibo(facade);

        Self {
            cubemap,
            irradiance_map,
            prefiltered_map,
            brdf_integration,
            vbo,
            ibo,
        }
    }

    pub fn irradiance_map(&self) -> Sampler<Cubemap> {
        self
            .irradiance_map
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn prefiltered_map(&self) -> Sampler<Cubemap> {
        self
            .prefiltered_map
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn brdf_integration(&self) -> Sampler<Texture2d> {
        self
            .brdf_integration
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn from_cubemap<F: ?Sized>(
        facade: &F,
        cubemap: Cubemap,
        irradiance_program: Program,
        prefiltered_program: Program,
        prefiltered_mipmap_count: u32,
        brdf_integration_program: Program,
        brdf_integration_dimesions: (u32, u32),
    ) -> Self
    where
        F: Facade,
    {
        let vbo = Self::make_vbo(facade);
        let ibo = Self::make_ibo(facade);

        let irradiance_map = Self::generate_irradiance_map(facade, &cubemap, irradiance_program);
        let prefiltered_map = Self::generate_prefiltered_map(
            facade,
            &cubemap,
            prefiltered_program,
            prefiltered_mipmap_count,
        );
        let brdf_integration = Self::generate_brdf_integration(
            facade,
            brdf_integration_program,
            brdf_integration_dimesions,
        );
        Self {
            cubemap,
            irradiance_map,
            prefiltered_map,
            brdf_integration,
            vbo,
            ibo,
        }
    }

    pub fn cubemap_framebuffers<'a, F: ?Sized>(
        facade: &F,
        cubemap: &'a Cubemap,
    ) -> [SimpleFrameBuffer<'a>; 6]
    where
        F: Facade,
    {
        Self::cubemap_mip_framebuffers(facade, cubemap, 0)
    }

    pub fn cubemap_mip_framebuffers<'a, F: ?Sized>(
        facade: &F,
        cubemap: &'a Cubemap,
        mip_level: u32,
    ) -> [SimpleFrameBuffer<'a>; 6]
    where
        F: Facade,
    {
        [
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::PositiveX),
            )
            .unwrap(),
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::NegativeX),
            )
            .unwrap(),
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::PositiveY),
            )
            .unwrap(),
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::NegativeY),
            )
            .unwrap(),
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::PositiveZ),
            )
            .unwrap(),
            glium::framebuffer::SimpleFrameBuffer::new(
                facade,
                cubemap
                    .mipmap(mip_level)
                    .unwrap()
                    .image(CubeLayer::NegativeZ),
            )
            .unwrap(),
        ]
    }

    pub fn generate_irradiance_map<F: ?Sized>(
        facade: &F,
        cubemap: &Cubemap,
        irradiance_program: Program,
    ) -> Cubemap
    where
        F: Facade,
    {
        let vbo = Self::make_vbo(facade);
        let ibo = Self::make_ibo(facade);

        let float_format = glium::texture::UncompressedFloatFormat::F32F32F32;
        let mipmaps_option = glium::texture::MipmapsOption::EmptyMipmaps;

        let irradiance_map =
            Cubemap::empty_with_format(facade, float_format, mipmaps_option, cubemap.height())
                .unwrap();

        let width = 32;
        let height = 32;

        let rect = glium::Rect {
            left: 0,
            bottom: 0,
            width,
            height,
        };

        let irradiance_renderbuffers = [
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
            RenderBuffer::new(facade, float_format, width, height).unwrap(),
        ];

        let mut frame_buffers = [
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[0]).unwrap(),
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[1]).unwrap(),
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[2]).unwrap(),
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[3]).unwrap(),
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[4]).unwrap(),
            SimpleFrameBuffer::new(facade, &irradiance_renderbuffers[5]).unwrap(),
        ];

        let projection_matrix = Self::cubemap_projection_matrix();
        let view_matrices = Self::cubemap_view_matrices();

        let draw_parameters = glium::DrawParameters {
            viewport: Some(rect),
            ..Default::default()
        };

        for i in 0..view_matrices.len() {
            let uniforms = uniform!(
                skybox: cubemap.sampled(),
                projection_matrix: projection_matrix,
                view_matrix: view_matrices[i]
            );
            frame_buffers[i]
                .draw(&vbo, &ibo, &irradiance_program, &uniforms, &draw_parameters)
                .unwrap();
        }

        let cubemap_frame_buffers = Self::cubemap_framebuffers(facade, &irradiance_map);

        let blit_rect = BlitTarget {
            left: 0,
            bottom: 0,
            width: 2048,
            height: 2048,
        };

        for i in 0..frame_buffers.len() {
            // blit_color instead of fill is necessary because of different resolutions, I believe
            frame_buffers[i].blit_color(
                &rect,
                &cubemap_frame_buffers[i],
                &blit_rect,
                MagnifySamplerFilter::Linear,
            );
        }

        unsafe {
            irradiance_map.generate_mipmaps();
        }

        irradiance_map
    }

    pub fn generate_prefiltered_map<F: ?Sized>(
        facade: &F,
        cubemap: &Cubemap,
        prefiltered_program: Program,
        mipmap_count: u32,
    ) -> Cubemap
    where
        F: Facade,
    {
        let mipmap_dimensions = (2u32.pow(mipmap_count - 1), 2u32.pow(mipmap_count - 1));
        let vbo = Self::make_vbo(facade);
        let ibo = Self::make_ibo(facade);

        let float_format = glium::texture::UncompressedFloatFormat::F32F32F32;
        let mipmaps_option = glium::texture::MipmapsOption::EmptyMipmaps;

        let prefiltered_map =
            Cubemap::empty_with_format(facade, float_format, mipmaps_option, mipmap_dimensions.0)
                .unwrap();

        let projection_matrix = Self::cubemap_projection_matrix();
        let view_matrices = Self::cubemap_view_matrices();

        for mip_level in 0..mipmap_count {
            let mipmap_width: u32 = mipmap_dimensions.0 / 2u32.pow(mip_level);
            let mipmap_height: u32 = mipmap_width;

            let roughness = mip_level as f32 / (mipmap_count as f32 - 1.0);
            let prefiltered_map_frame_buffers =
                Self::cubemap_mip_framebuffers(facade, &prefiltered_map, mip_level);

            let prefiltered_renderbuffers = [
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
                RenderBuffer::new(facade, float_format, mipmap_width, mipmap_height).unwrap(),
            ];

            // let rect = glium::Rect {
            //     left: 0,
            //     bottom: 0,
            //     width: mipmap_width,
            //     height: mipmap_height,
            // };

            let draw_parameters = glium::DrawParameters {
                // viewport: Some(rect),
                ..Default::default()
            };

            for view_matrix_index in 0..view_matrices.len() {
                let uniforms = uniform! {
                    skybox: cubemap.sampled(),
                    roughness: roughness,
                    projection_matrix: projection_matrix,
                    view_matrix: view_matrices[view_matrix_index]
                };
                let mut temp_frame_buffer =
                    SimpleFrameBuffer::new(facade, &prefiltered_renderbuffers[view_matrix_index])
                        .unwrap();
                temp_frame_buffer
                    .draw(
                        &vbo,
                        &ibo,
                        &prefiltered_program,
                        &uniforms,
                        &draw_parameters,
                    )
                    .unwrap();
                temp_frame_buffer.fill(
                    &prefiltered_map_frame_buffers[view_matrix_index],
                    MagnifySamplerFilter::Linear,
                );
            }
        }

        // Shouldn't need to generate mipmaps because they are already all made
        prefiltered_map
    }

    pub fn generate_brdf_integration<F: ?Sized>(
        facade: &F,
        brdf_integration_program: Program,
        dimensions: (u32, u32),
    ) -> Texture2d
    where
        F: Facade,
    {
        let vbo = QuadVertex::texture_quad_vbo(facade);
        let ibo = QuadVertex::texture_quad_ibo(facade);

        let float_format = glium::texture::UncompressedFloatFormat::F32F32F32F32;
        let mipmaps_option = glium::texture::MipmapsOption::NoMipmap;

        let texture = Texture2d::empty_with_format(
            facade,
            float_format,
            mipmaps_option,
            dimensions.0,
            dimensions.1,
        )
        .unwrap();

        let texture_framebuffer = SimpleFrameBuffer::new(facade, &texture).unwrap();

        let render_buffer =
            RenderBuffer::new(facade, float_format, dimensions.0, dimensions.1).unwrap();

        let mut framebuffer = SimpleFrameBuffer::new(facade, &render_buffer).unwrap();

        let rect = glium::Rect {
            left: 0,
            bottom: 0,
            width: dimensions.0,
            height: dimensions.1,
        };

        let draw_parameters = glium::DrawParameters {
            viewport: Some(rect),
            ..Default::default()
        };

        framebuffer
            .draw(
                &vbo,
                &ibo,
                &brdf_integration_program,
                &glium::uniforms::EmptyUniforms,
                &draw_parameters,
            )
            .unwrap();

        framebuffer.fill(&texture_framebuffer, MagnifySamplerFilter::Linear);
        texture
    }

    pub fn make_vbo<F: ?Sized>(facade: &F) -> VertexBuffer<SkyboxVertex>
    where
        F: Facade,
    {
        let vertices: [SkyboxVertex; 8] = [
            SkyboxVertex::new([1.0, 1.0, 1.0]),    //0
            SkyboxVertex::new([-1.0, 1.0, 1.0]),   //1
            SkyboxVertex::new([1.0, -1.0, 1.0]),   //2
            SkyboxVertex::new([1.0, 1.0, -1.0]),   //3
            SkyboxVertex::new([-1.0, -1.0, 1.0]),  //4
            SkyboxVertex::new([-1.0, 1.0, -1.0]),  //5
            SkyboxVertex::new([1.0, -1.0, -1.0]),  //6
            SkyboxVertex::new([-1.0, -1.0, -1.0]), //7
        ];

        glium::VertexBuffer::new(facade, &vertices).unwrap()
    }

    pub fn make_ibo<F: ?Sized>(facade: &F) -> IndexBuffer<u32>
    where
        F: Facade,
    {
        let indices: [u32; 36] = [
            1, 3, 0, //0, 3, 1,
            3, 1, 5, //5, 1, 3,
            4, 5, 1, //1, 5, 4,
            5, 4, 7, //7, 4, 5,
            2, 7, 4, //4, 7, 2,
            7, 2, 6, //6, 2, 7,
            0, 6, 2, //2, 6, 0,
            6, 0, 3, //3, 0, 6,
            1, 2, 4, //4, 2, 1,
            2, 1, 0, //0, 1, 2,
            5, 6, 3, //3, 6, 5,
            6, 5, 7, //7, 5, 6,
        ];

        glium::IndexBuffer::new(facade, glium::index::PrimitiveType::TrianglesList, &indices)
            .unwrap()
    }

    pub fn cubemap_view_matrices() -> [[[f32; 4]; 4]; 6] {
        let view_eye = Point3::new(0.0, 0.0, 0.0);
        [
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(1.0, 0.0, 0.0),
                &Vector3::new(0.0, -1.0, 0.0),
            )
            .to_homogeneous()
            .into(),
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(-1.0, 0.0, 0.0),
                &Vector3::new(0.0, -1.0, 0.0),
            )
            .to_homogeneous()
            .into(),
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(0.0, 1.0, 0.0),
                &Vector3::new(0.0, 0.0, 1.0),
            )
            .to_homogeneous()
            .into(),
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(0.0, -1.0, 0.0),
                &Vector3::new(0.0, 0.0, -1.0),
            )
            .to_homogeneous()
            .into(),
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(0.0, 0.0, 1.0),
                &Vector3::new(0.0, -1.0, 0.0),
            )
            .to_homogeneous()
            .into(),
            Isometry3::look_at_rh(
                &view_eye,
                &Point3::new(0.0, 0.0, -1.0),
                &Vector3::new(0.0, -1.0, 0.0),
            )
            .to_homogeneous()
            .into(),
        ]
    }

    pub fn cubemap_projection_matrix() -> [[f32; 4]; 4] {
        Perspective3::new(1.0, 1.5707963267949, 0.1, 10.0)
            .to_homogeneous()
            .into()
    }
}
