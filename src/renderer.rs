use crate::camera::Camera;
use crate::light::Light;
use crate::model::{Model, ModelHandle};
use crate::map::Map;
use crate::skybox::Skybox;
use glium::backend::Facade;
use glium::draw_parameters;
use glium::glutin::dpi::PhysicalSize;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::uniforms::UniformBuffer;
use glium::Display;
use glium::Frame;
use glium::{DrawParameters, Program, Surface};

use legion::*;

const MAX_LIGHT_COUNT: usize = 512;

pub struct RendererState {
    pub camera: Camera,
    draw_parameters: DrawParameters<'static>,
    model_program: Program,
    skybox_program: Program,
    lights: [Light; MAX_LIGHT_COUNT],
    models: Vec<Model>,
}

impl RendererState {
    pub fn new(camera: Camera, model_program: Program, skybox_program: Program) -> Self {
        let draw_parameters = glium::DrawParameters {
            backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
            depth: glium::Depth {
                test: draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut lights: [Light; MAX_LIGHT_COUNT] = [Light::default(); MAX_LIGHT_COUNT];
        lights[0] = Light::new(glm::vec3(-0.5, 0.1, -0.5), None);
        let models = Vec::new();

        Self {
            camera,
            draw_parameters,
            model_program,
            skybox_program,
            lights,
            models,
        }
    }

    pub fn draw_model<F, S>(
        &self,
        facade: &F,
        surface: &mut S,
        model: &ModelHandle,
        skybox: &Skybox,
    ) where
        F: Facade,
        S: Surface,
    {
        let model = self.get_model(model);
        let mut light_colours: [[f32; 3]; MAX_LIGHT_COUNT] =
            [[std::f32::MAX, std::f32::MAX, std::f32::MAX]; MAX_LIGHT_COUNT];
        let mut light_positions: [[f32; 3]; MAX_LIGHT_COUNT] =
            [[std::f32::MAX, std::f32::MAX, std::f32::MAX]; MAX_LIGHT_COUNT];

        for i in 0..self.lights.len() {
            light_positions[i] = self.lights[i].position;
            light_colours[i] = self.lights[i].colour;
        }

        let light_positions = UniformBuffer::new(facade, light_positions).unwrap();
        let light_colours = UniformBuffer::new(facade, light_colours).unwrap();

        for mesh in model.meshes.iter() {
            for primitive in mesh.primitives.iter() {
                let material = &model.materials[primitive.material_index];
                let uniforms = uniform! {
                    irradiance_map : skybox.irradiance_map.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                    prefiltered_map : skybox.prefiltered_map.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                    brdf_integration : skybox.brdf_integration.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                    model_matrix : mesh.transformation(),
                    view_matrix : self.camera.view_matrix(),
                    projection_matrix : self.camera.projection_matrix(),
                    diffuse_map : material.diffuse_map(),
                    occlusion_roughness_metal_map : material.orm_map(),
                    normal_map : material.normal_map(),
                    view_position : self.camera.position(),
                    light_positions : &light_positions,
                    light_colours : &light_colours
                };

                surface
                    .draw(
                        &primitive.vbo,
                        &primitive.ibo,
                        &self.model_program,
                        &uniforms,
                        &self.draw_parameters,
                    )
                    .unwrap();
            }
        }
    }

    pub fn push_model(&mut self, model: Model) -> ModelHandle {
        self.models.push(model);
        ModelHandle::new(self.models.len() - 1)
    }

    pub fn get_model(&self, model_handle: &ModelHandle) -> &Model {
        &self.models[model_handle.index()]
    }

    pub fn get_mut_model(&mut self, model_handle: &ModelHandle) -> &mut Model {
        &mut self.models[model_handle.index()]
    }

    //    pub fn draw_skybox<S>(&self, surface: &mut S, skybox: &Skybox)
    //    where
    //        S: Surface,
    //    {
    //        let uniforms = uniform! {
    //            view_matrix : self.camera.skybox_view_matrix(),
    //            projection_matrix : self.camera.projection_matrix(),
    //            cubemap : skybox.cubemap.sampled()
    //        };
    //
    //        surface
    //            .draw(
    //                &skybox.vbo,
    //                &skybox.ibo,
    //                &self.skybox_program,
    //                &uniforms,
    //                &self.draw_parameters,
    //            )
    //            .unwrap();
    //    }
}

#[system(for_each)]
pub fn render_models(
    model_handle: &ModelHandle,
    #[resource] rs: &mut RendererState,
    #[resource] ds: &mut DisplayState,
    #[resource] skybox: &Skybox,
) {
    rs.draw_model(&ds.display, ds.target.as_mut().unwrap(), model_handle, skybox);
}

#[system]
pub fn render_map(
    #[resource] map: &Map,
    #[resource] rs: &mut RendererState,
    #[resource] ds: &mut DisplayState,
    #[resource] skybox: &Skybox,
) {
    rs.draw_model(&ds.display, ds.target.as_mut().unwrap(), &map.model, skybox);
}

#[system]
pub fn render_skybox(
    #[resource] rs: &RendererState,
    #[resource] ds: &mut DisplayState,
    #[resource] skybox: &Skybox,
) {
    let uniforms = uniform! {
        view_matrix : rs.camera.skybox_view_matrix(),
        projection_matrix : rs.camera.projection_matrix(),
        cubemap : skybox.cubemap.sampled()
    };

    ds.target
        .as_mut()
        .unwrap()
        .draw(
            &skybox.vbo,
            &skybox.ibo,
            &rs.skybox_program,
            &uniforms,
            &rs.draw_parameters,
        )
        .unwrap();
}

#[system]
pub fn update_target(#[resource] ds: &mut DisplayState) {
    ds.update();
}

#[system]
pub fn close_display(#[resource] ds: &mut DisplayState) {
    ds.close_display();
}

/// This struct holds reference to the OpenGL context (display) and the current frame which should
/// be drawn to.
pub struct DisplayState {
    pub target: Option<Frame>,
    pub display: Display,
}

impl DisplayState {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let wb = WindowBuilder::new()
            .with_title("PBR RendererState")
            .with_inner_size(PhysicalSize::new(1920, 1440));
        let cb = ContextBuilder::new().with_depth_buffer(24);
        let display = Display::new(wb, cb, &event_loop).unwrap();
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        let target = Some(target);

        Self { target, display }
    }

    pub fn update(&mut self) {
        // Discard old frame
        // self.target.take().unwrap().finish().unwrap();
        let target = self.target.take().unwrap().finish().unwrap();
        drop(target);

        // Get a new one and clear it
        let mut new_target = self.display.draw();
        new_target.clear_color_and_depth((0.0, 1.0, 0.0, 1.0), 1.0);
        self.target = Some(new_target);
    }

    pub fn close_display(&mut self) {
        // Discard final frame
        self.target.take().unwrap().finish().unwrap();
    }
}
