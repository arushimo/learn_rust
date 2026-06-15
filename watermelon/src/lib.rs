use rapier2d::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::mpsc;
use wasm_bindgen::prelude::*;

const STAGE_WIDTH: f32 = 480.0;
const STAGE_HEIGHT: f32 = 640.0;
const WALL_THICKNESS: f32 = 24.0;
const DROP_Y: f32 = 36.0;
const MAX_LEVEL: u8 = 5;
const FRUIT_DENSITY: f32 = 0.9;
const FRUIT_RESTITUTION: f32 = 0.35;
const FRUIT_FRICTION: f32 = 0.7;

fn fruit_radius(level: u8) -> f32 {
    match level.min(MAX_LEVEL) {
        0 => 18.0,
        1 => 24.0,
        2 => 31.0,
        3 => 40.0,
        4 => 52.0,
        _ => 68.0,
    }
}

fn encode_level(level: u8) -> u128 {
    // 0 is reserved for walls, so fruit levels are stored as level + 1.
    u128::from(level) + 1
}

fn decode_level(user_data: u128) -> Option<u8> {
    if user_data == 0 {
        None
    } else {
        Some((user_data - 1) as u8)
    }
}

#[derive(Serialize)]
pub struct FruitView {
    x: f32,
    y: f32,
    radius: f32,
    level: u8,
}

#[wasm_bindgen]
pub struct GameState {
    gravity: Vector,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    collision_receiver: mpsc::Receiver<CollisionEvent>,
    contact_force_receiver: mpsc::Receiver<ContactForceEvent>,
    event_handler: ChannelEventCollector,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        let (collision_sender, collision_receiver) = mpsc::channel();
        let (contact_force_sender, contact_force_receiver) = mpsc::channel();
        let mut state = GameState {
            gravity: Vector::new(0.0, 980.0),
            integration_parameters: IntegrationParameters {
                dt: 1.0 / 60.0,
                ..IntegrationParameters::default()
            },
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            collision_receiver,
            contact_force_receiver,
            event_handler: ChannelEventCollector::new(collision_sender, contact_force_sender),
        };

        state.add_walls();
        state
    }

    pub fn add_fruit(&mut self, x: f32, y: f32, level: u8) {
        let level = level.min(MAX_LEVEL);
        self.spawn_fruit(x, y, level);
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &self.event_handler,
        );

        self.handle_merge_events();
        self.drain_contact_force_events();
    }

    pub fn get_fruits(&self) -> Result<JsValue, JsValue> {
        let fruits: Vec<FruitView> = self
            .colliders
            .iter()
            .filter_map(|(_, collider)| {
                let level = decode_level(collider.user_data)?;
                let body_handle = collider.parent()?;
                let body = self.bodies.get(body_handle)?;
                let position = body.translation();

                Some(FruitView {
                    x: position.x,
                    y: position.y,
                    radius: fruit_radius(level),
                    level,
                })
            })
            .collect();

        serde_wasm_bindgen::to_value(&fruits).map_err(|err| JsValue::from_str(&err.to_string()))
    }
}

impl GameState {
    fn add_walls(&mut self) {
        self.add_wall(
            STAGE_WIDTH / 2.0,
            STAGE_HEIGHT + WALL_THICKNESS / 2.0,
            STAGE_WIDTH / 2.0,
            WALL_THICKNESS / 2.0,
        );
        self.add_wall(
            -WALL_THICKNESS / 2.0,
            STAGE_HEIGHT / 2.0,
            WALL_THICKNESS / 2.0,
            STAGE_HEIGHT / 2.0,
        );
        self.add_wall(
            STAGE_WIDTH + WALL_THICKNESS / 2.0,
            STAGE_HEIGHT / 2.0,
            WALL_THICKNESS / 2.0,
            STAGE_HEIGHT / 2.0,
        );
    }

    fn add_wall(&mut self, x: f32, y: f32, half_width: f32, half_height: f32) {
        let body = RigidBodyBuilder::fixed()
            .translation(Vector::new(x, y))
            .build();
        let body_handle = self.bodies.insert(body);
        let collider = ColliderBuilder::cuboid(half_width, half_height)
            .friction(0.9)
            .restitution(0.2)
            .build();
        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies);
    }

    fn spawn_fruit(&mut self, x: f32, y: f32, level: u8) {
        let radius = fruit_radius(level);
        let clamped_x = x.clamp(radius, STAGE_WIDTH - radius);
        let clamped_y = y.max(DROP_Y);
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(clamped_x, clamped_y))
            .ccd_enabled(true)
            .build();
        let body_handle = self.bodies.insert(body);
        let collider = ColliderBuilder::ball(radius)
            .density(FRUIT_DENSITY)
            .restitution(FRUIT_RESTITUTION)
            .friction(FRUIT_FRICTION)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .user_data(encode_level(level))
            .build();
        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies);
    }

    fn handle_merge_events(&mut self) {
        let mut merge_requests = Vec::new();

        while let Ok(event) = self.collision_receiver.try_recv() {
            if let CollisionEvent::Started(collider_a, collider_b, _) = event {
                if let Some(request) = self.build_merge_request(collider_a, collider_b) {
                    merge_requests.push(request);
                }
            }
        }

        let mut consumed_colliders = HashSet::new();
        for request in merge_requests {
            if !consumed_colliders.insert(request.collider_a)
                || !consumed_colliders.insert(request.collider_b)
            {
                continue;
            }

            self.remove_body(request.body_a);
            self.remove_body(request.body_b);

            if request.level < MAX_LEVEL {
                self.spawn_fruit(request.x, request.y, request.level + 1);
            }
        }
    }

    fn build_merge_request(
        &self,
        collider_a: ColliderHandle,
        collider_b: ColliderHandle,
    ) -> Option<MergeRequest> {
        let collider_a_ref = self.colliders.get(collider_a)?;
        let collider_b_ref = self.colliders.get(collider_b)?;
        let level_a = decode_level(collider_a_ref.user_data)?;
        let level_b = decode_level(collider_b_ref.user_data)?;

        if level_a != level_b {
            return None;
        }

        let body_a = collider_a_ref.parent()?;
        let body_b = collider_b_ref.parent()?;
        let pos_a = self.bodies.get(body_a)?.translation();
        let pos_b = self.bodies.get(body_b)?.translation();

        Some(MergeRequest {
            collider_a,
            collider_b,
            body_a,
            body_b,
            level: level_a,
            x: (pos_a.x + pos_b.x) * 0.5,
            y: (pos_a.y + pos_b.y) * 0.5,
        })
    }

    fn remove_body(&mut self, handle: RigidBodyHandle) {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    fn drain_contact_force_events(&mut self) {
        while self.contact_force_receiver.try_recv().is_ok() {}
    }
}

struct MergeRequest {
    collider_a: ColliderHandle,
    collider_b: ColliderHandle,
    body_a: RigidBodyHandle,
    body_b: RigidBodyHandle,
    level: u8,
    x: f32,
    y: f32,
}
