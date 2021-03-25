extern crate glium;

extern crate nalgebra_glm as glm;

use glium::glutin;
use learning_glium::camera::Camera;
use learning_glium::import;
use learning_glium::map::Map;
use learning_glium::physics::*;
use learning_glium::renderer::*;
use legion::*;

use rapier3d::dynamics::*;
use rapier3d::geometry::*;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let mut world = World::default();
    let mut resources = Resources::default();
    let mut schedule = Schedule::builder()
        .add_system(update_physics_system())
        .flush()
        .add_thread_local(update_model_transform_system())
        .flush()
        .add_thread_local(render_models_system())
        .flush()
        .add_thread_local(render_map_system())
        .flush()
        .add_thread_local(render_skybox_system())
        .flush()
        .add_thread_local(update_target_system())
        .flush()
        .build();

    let display_state = DisplayState::new(&event_loop);
    let mut physics_state = PhysicsState::default();
    let display = &display_state.display;

    // Used to draw with PBR workflow
    let vertex_shader_src = include_str!("shaders/entity.vs");
    let fragment_shader_src = include_str!("shaders/entity.fs");
    let model_program = import::load_program(display, vertex_shader_src, fragment_shader_src);

    // Used to draw entire skybox
    let cubemap_vertex_src = include_str!("shaders/skybox.vs");
    let cubemap_fragment_src = include_str!("shaders/skybox.fs");
    let skybox_program = import::load_program(display, cubemap_vertex_src, cubemap_fragment_src);

    // Used to convert .hdr files into cubemaps
    let hdr_vertex_src = include_str!("shaders/skybox.vs");
    let hdr_fragment_src = include_str!("shaders/hdr.fs");
    let hdr_program = import::load_program(display, hdr_vertex_src, hdr_fragment_src);

    // Used to generate irradiance map
    let irradiance_vertex_src = include_str!("shaders/skybox.vs");
    let irradiance_fragment_src = include_str!("shaders/irradiance.fs");
    let irradiance_program =
        import::load_program(display, irradiance_vertex_src, irradiance_fragment_src);

    // Used to generetae prefiltered maps
    // Reflection LOD vs roughness stored in mipmaps
    let prefiltered_vertex_src = include_str!("shaders/skybox.vs");
    let prefiltered_fragment_src = include_str!("shaders/prefiltered.fs");
    let prefiltered_program =
        import::load_program(display, prefiltered_vertex_src, prefiltered_fragment_src);

    // Used to generate the brdf look up texture
    let brdf_integration_vertex_src = include_str!("shaders/brdf_integration.vs");
    let brdf_integration_fragment_src = include_str!("shaders/brdf_integration.fs");
    let brdf_integration_program = import::load_program(
        display,
        brdf_integration_vertex_src,
        brdf_integration_fragment_src,
    );

    //    let skybox_path = "assets/skybox";
    //    let skybox = import::load_skybox(&display, &skybox_path);
    let skybox_path = "assets/reinforced_concrete_01_2k.hdr";
    let skybox = import::load_skybox_from_hdr(
        display,
        &skybox_path,
        hdr_program,
        irradiance_program,
        prefiltered_program,
        brdf_integration_program,
    );

    let mut renderer = RendererState::new(Camera::default(), model_program, skybox_program);

    let akm_gltf_path = "assets/AKM_glTF";
    let akm_model_handle = import::model_from_gltf(display, &mut renderer, &akm_gltf_path).unwrap();

    let akm_rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(-1.0, 10.0, -3.0)
        .mass(1.0)
        .build();
    let akm_collider = ColliderBuilder::cuboid(0.5, 0.2, 0.5)
        .restitution(0.3)
        .build();
    let akm_physics = Physics::new(&mut physics_state, akm_rigid_body, akm_collider);

    let map_gltf_path = "assets/block_map_debug";
    let map_model_handle = import::model_from_gltf(display, &mut renderer, &map_gltf_path).unwrap();
    let map = Map::from_model(map_model_handle, &renderer, &mut physics_state);

    let _akm_entity: Entity = world.push((akm_physics, akm_model_handle));

    //TODO Maybe make it so that resources doesn't have to have everything inserted at the end?
    resources.insert(renderer);
    resources.insert(physics_state);
    resources.insert(display_state);
    resources.insert(skybox);
    resources.insert(map);
    // let y_rotation = 0.0f32;
    //    let mut eye_angle = 0.0f32;
    //
    //    let mut current_time = std::time::Instant::now();
    //
    //
    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    resources.get_mut::<DisplayState>().unwrap().close_display();
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        schedule.execute(&mut world, &mut resources);

        //        let display = &resources.get::<DisplayResource>().unwrap().display;
        //        let mut target = display.draw();
        //
        //        let model_test = &resources.get::<Renderer>().unwrap().get_model(&akm_model_handle);
        //
        //        let skybox_test = &resources.get::<Skybox>().unwrap();
        //
        //        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        //        renderer.draw_model(&display, &mut target, &akm_model_handle, &skybox_test);
        //        renderer.draw_skybox(&mut target, &skybox_test);
        //        target.finish().unwrap();

        //        //        y_rotation += 0.0065;
        //        eye_angle += 0.0065;
        //
        //
        //        let new_time = std::time::Instant::now();
        //        if new_time - current_time > std::time::Duration::from_nanos(20_000_000) {
        //            println!("{:?}", (new_time - current_time));
        //        }
        //        current_time = std::time::Instant::now();
        //
        //        let view_eye = glm::vec3(eye_angle.cos(), 0.0, eye_angle.sin());
        //        let view_center = glm::vec3(0.0, 0.0, 0.0);
        //        let view_up = glm::vec3(0.0, 1.0, 0.0);
        //        let view_matrix = glm::look_at(&view_eye, &view_center, &view_up);
        //
        //        let projection_aspect = 4.0 / 3.0;
        //        let projection_fov = 1.047198;
        //        let projection_near = 0.1;
        //        let projection_far = 100.0;
        //        let projection_matrix = glm::perspective(
        //            projection_aspect,
        //            projection_fov,
        //            projection_near,
        //            projection_far,
        //        );
        //
        //        let mut camera = Camera::new(Some(view_matrix), Some(projection_matrix));
        //        camera.position = view_eye;
        //        renderer.camera = camera;
        //
        //        let akm_rotation = glm::vec3(0.0, y_rotation, 0.0);
        //        let akm_scale = glm::vec3(1.5f32, 1.5f32, 1.5f32);
        //        let akm_rotation_matrix = glm::rotation(1.0 * y_rotation, &akm_rotation);
        //        let akm_final_transformation = glm::scale(&akm_rotation_matrix, &akm_scale);
        //        for mesh in akm_entity.model.meshes.iter_mut() {
        //            mesh.transformation_matrix = akm_final_transformation;
        //        }
        //
        //        let mut target = display.draw();
        //        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        //        renderer.draw_model(&display, &mut target, &akm_entity.model, &skybox);
        //        renderer.draw_skybox(&mut target, &skybox);
        //        target.finish().unwrap();
    });
    //debugging();

    // Put in main loop
}

// Debugging stuff
//use glium::uniform;
//use learning_glium::matrix;
//use learning_glium::skybox::Skybox;
//#[allow(dead_code)]
//fn debugging() {
//    let event_loop = glutin::event_loop::EventLoop::new();
//    let wb = glutin::window::WindowBuilder::new().with_title("PBR Renderer");
//    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
//    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
//
//    let test_vertex_shader_src = include_str!("shaders/texture_debug.vs");
//    let test_fragment_shader_src = include_str!("shaders/texture_debug.fs");
//    let test_program =
//        import::load_program(&display, test_vertex_shader_src, test_fragment_shader_src);
//
//    let brdf_integration_vertex_src = include_str!("shaders/brdf_integration.vs");
//    let brdf_integration_fragment_src = include_str!("shaders/brdf_integration.fs");
//    let brdf_integration_program = import::load_program(
//        &display,
//        brdf_integration_vertex_src,
//        brdf_integration_fragment_src,
//    );
//    let test_texture =
//        Skybox::generate_brdf_integration(&display, brdf_integration_program, (512, 512));
//    event_loop.run(move |event, _, control_flow| {
//        match event {
//            glutin::event::Event::WindowEvent { event, .. } => match event {
//                glutin::event::WindowEvent::CloseRequested => {
//                    *control_flow = glutin::event_loop::ControlFlow::Exit;
//                    return;
//                }
//                _ => return,
//            },
//            glutin::event::Event::NewEvents(cause) => match cause {
//                glutin::event::StartCause::ResumeTimeReached { .. } => (),
//                glutin::event::StartCause::Init => (),
//                _ => return,
//            },
//            _ => return,
//        }
//
//        let mut target = display.draw();
//        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
//        let projection_matrix = glm::perspective(1.0, 1.5707963267949, 0.1, 10.0).into();
//        let view_matrix = glm::look_at(
//            &glm::vec3(0.0, 0.0, 0.0),
//            &glm::vec3(0.0, 0.0, 1.0),
//            &glm::vec3(0.0, 1.0, 0.0),
//        );
//
//        let uniforms = uniform! {
//            projection_matrix: matrix::as_array(&projection_matrix),
//            view_matrix: matrix::as_array(&view_matrix),
//            test_texture: &test_texture,
//        };
//
//        let vbo = learning_glium::vertex::QuadVertex::texture_quad_vbo(&display);
//        let ibo = learning_glium::vertex::QuadVertex::texture_quad_ibo(&display);
//
//        let draw_parameters = glium::draw_parameters::DrawParameters {
//            //            viewport: Some(Rect {
//            //                bottom: 0,
//            //                left: 0,
//            //                width: 512,
//            //                height: 512,
//            //            }),
//            ..Default::default()
//        };
//
//        target
//            .draw(&vbo, &ibo, &test_program, &uniforms, &draw_parameters)
//            .unwrap();
//
//        target.finish().unwrap();
//    });
//}
