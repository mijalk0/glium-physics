use glium::texture::{SrgbTexture2d, Texture2d};
use glium::uniforms::Sampler;

pub struct Material {
    pub diffuse_map: SrgbTexture2d,
    pub occlusion_roughness_metal_map: Texture2d,
    pub normal_map: Texture2d,
}

impl Material {
    pub fn new(
        diffuse_map: SrgbTexture2d,
        occlusion_roughness_metal_map: Texture2d,
        normal_map: Texture2d,
    ) -> Self { Self {
            diffuse_map,
            occlusion_roughness_metal_map,
            normal_map,
        }
    }

    pub fn diffuse_map(&self) -> Sampler<SrgbTexture2d> {
        self.diffuse_map
            .sampled()
            .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
//            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }

    pub fn orm_map(&self) -> Sampler<Texture2d> {
        self.occlusion_roughness_metal_map
            .sampled()
            .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
//            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }
    pub fn normal_map(&self) -> Sampler<Texture2d> {
        self.normal_map
            .sampled()
            .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat)
//            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
    }
}
