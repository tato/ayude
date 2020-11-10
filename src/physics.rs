use rapier3d::{
    dynamics::IntegrationParameters, dynamics::JointSet, dynamics::RigidBodySet,
    geometry::BroadPhase, geometry::ColliderSet, geometry::NarrowPhase, pipeline::PhysicsPipeline,
};

pub struct PhysicsState {
    fixed_step_accumulator: f32,
    physics_pipeline: PhysicsPipeline,
    gravity: rapier3d::na::Vector3<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: JointSet,
}

impl PhysicsState {
    pub fn new() -> Self {
        let physics_pipeline = PhysicsPipeline::new();
        let gravity = rapier3d::na::Vector3::new(0.0, -9.81, 0.0);
        let integration_parameters = IntegrationParameters::default();
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let bodies = RigidBodySet::new();
        let colliders = ColliderSet::new();
        let joints = JointSet::new();
        PhysicsState {
            fixed_step_accumulator: 0.0,
            physics_pipeline,
            gravity,
            integration_parameters,
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            joints,
        }
    }

    pub fn step(&mut self) {
        let event_handler = ();
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &event_handler,
        );
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.fixed_step_accumulator += delta.as_secs_f32();
        while self.fixed_step_accumulator > self.integration_parameters.dt() {
            self.fixed_step_accumulator -= self.integration_parameters.dt();
            self.step();
        }
    }
}
