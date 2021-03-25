use crate::model::ModelHandle;
use crate::renderer::RendererState;
use crossbeam::channel::Receiver;
use legion::*;
use rapier3d::dynamics::{
    IntegrationParameters, JointSet, RigidBody, RigidBodyHandle, RigidBodySet,
};
use rapier3d::geometry::{
    BroadPhase, Collider, ColliderHandle, ColliderSet, ContactEvent, IntersectionEvent, NarrowPhase,
};
use rapier3d::na::Vector3;
use rapier3d::pipeline::{ChannelEventCollector, PhysicsPipeline};

use na::{Isometry3, Translation3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Physics {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

impl Physics {
    pub fn new(ps: &mut PhysicsState, rigid_body: RigidBody, collider: Collider) -> Self {
        let rigid_body_handle = ps.bodies.insert(rigid_body);
        let collider_handle = ps
            .colliders
            .insert(collider, rigid_body_handle, &mut ps.bodies);
        Self {
            rigid_body_handle,
            collider_handle,
        }
    }
}

pub struct PhysicsState {
    pub pipeline: PhysicsPipeline,
    pub gravity: Vector3<f32>,
    pub integration_parameters: IntegrationParameters,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    pub event_handler: ChannelEventCollector,
    pub intersection_recv: Receiver<IntersectionEvent>,
    pub contact_recv: Receiver<ContactEvent>,
    //    pub hooks: Box<dyn PhysicsHooks>,
}

impl Default for PhysicsState {
    fn default() -> Self {
        let (contact_send, contact_recv) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_recv) = crossbeam::channel::unbounded();

        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector3::y() * -3.00,
            integration_parameters: IntegrationParameters::default(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            event_handler: ChannelEventCollector::new(intersection_send, contact_send),
            intersection_recv,
            contact_recv,
            // hooks: Box::new::<>(())
        }
    }
}

#[system]
pub fn update_physics(#[resource] ps: &mut PhysicsState) {
    ps.pipeline.step(
        &ps.gravity,
        &ps.integration_parameters,
        &mut ps.broad_phase,
        &mut ps.narrow_phase,
        &mut ps.bodies,
        &mut ps.colliders,
        &mut ps.joints,
        None,
        None,
        // &*ps.hooks,
        &mut ps.event_handler,
    );

    // for (_, rigid_body) in ps.bodies.iter_mut() {
    //     if rigid_body.position().translation.vector.y < -1.2 {
    //         rigid_body.set_position(
    //             Isometry3::from_parts(
    //                 Translation3::from(Vector3::y() * 2.0),
    //                 rigid_body.position().rotation,
    //             ),
    //             true,
    //         );
    //     }
    //
    //     if rigid_body.linvel().y < -0.65 {
    //         rigid_body.set_linvel(Vector3::y() * -0.65, true);
    //     }
    // }
}

#[system(for_each)]
pub fn update_model_transform(
    model_handle: &ModelHandle,
    physics: &Physics,
    #[resource] renderer: &mut RendererState,
    #[resource] ps: &mut PhysicsState,
) {
    let model = renderer.get_mut_model(model_handle);

    for mesh in model.meshes.iter_mut() {
        // This might become based on colliders if a model has more than one collider
        let rigid_body = ps.bodies.get(physics.rigid_body_handle).unwrap();
        let physics_isometry = rigid_body.position();

        let new_isometry = mesh.base_isometry * *physics_isometry;
        mesh.update_isometry(new_isometry);
    }
}
