#![allow(unused_labels)]
use leptos::prelude::*;
use rapier2d::dynamics::{CCDSolver, ImpulseJointSet, IslandManager, MultibodyJointSet};
use rapier2d::geometry::{BroadPhaseBvh, NarrowPhase};
use rapier2d::prelude::*;
use std::sync::RwLock;

use crate::fun_drag::{BoxSettings, ContainerSize};

#[derive(Clone, Debug)]
pub struct WorldBox {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub position: RwSignal<(f32, f32)>, // for UI rendering
    pub animation_state: RwSignal<AnimationState>,
}

pub struct RapierContext {
    pub colliders: RwLock<ColliderSet>,
    pub rigid_bodies: RwLock<RigidBodySet>,
    pub pipeline: RwLock<PhysicsPipeline>,
    pub params: RwLock<IntegrationParameters>,
    pub hooks: RwLock<()>,
    pub islands: RwLock<IslandManager>,
    pub broad_phase: RwLock<BroadPhaseBvh>,
    pub narrow_phase: RwLock<NarrowPhase>,
    pub impulse_joints: RwLock<ImpulseJointSet>,
    pub multibody_joints: RwLock<MultibodyJointSet>,
    pub ccd_solver: RwLock<CCDSolver>,
}

impl RapierContext {
    pub fn new() -> Self {
        Self {
            colliders: RwLock::new(ColliderSet::new()),
            rigid_bodies: RwLock::new(RigidBodySet::new()),
            pipeline: RwLock::new(PhysicsPipeline::new()),
            params: RwLock::new(IntegrationParameters::default()),
            hooks: RwLock::new(()),
            islands: RwLock::new(IslandManager::new()),
            broad_phase: RwLock::new(BroadPhaseBvh::new()),
            narrow_phase: RwLock::new(NarrowPhase::new()),
            impulse_joints: RwLock::new(ImpulseJointSet::new()),
            multibody_joints: RwLock::new(MultibodyJointSet::new()),
            ccd_solver: RwLock::new(CCDSolver::new()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationState {
    ActivelyDragged,
    TouchingMouse,
    PassivelyMoving,
    Still,
}

#[derive(Clone, Debug)]
pub struct ActivelyMovingBoxes {
    pub box_ids: Vec<u32>,
}

// Collision detection only – no physics stepping.
pub fn check_if_colliding_with_another_box(
    colliders: &ColliderSet,
    position: (u32, u32),
    box_settings: &BoxSettings,
) -> bool {
    let new_left = position.0 as f32;
    let new_top = position.1 as f32;
    let new_right = new_left + box_settings.width as f32;
    let new_bottom = new_top + box_settings.height as f32;

    for (_, collider) in colliders.iter() {
        let Some(cuboid) = collider.shape().as_cuboid() else {
            continue;
        };
        let center = collider.position().translation;
        let existing_left = center.x - cuboid.half_extents.x;
        let existing_top = center.y - cuboid.half_extents.y;
        let existing_right = center.x + cuboid.half_extents.x;
        let existing_bottom = center.y + cuboid.half_extents.y;

        let overlapping = new_left < existing_right
            && new_right > existing_left
            && new_top < existing_bottom
            && new_bottom > existing_top;

        if overlapping {
            return true;
        }
    }
    false
}

pub fn update_world(
    rapier_ctx: ArcRwSignal<RapierContext>,
    mouse_pos: ReadSignal<(f32, f32)>,
    boxes: ReadSignal<Vec<WorldBox>>,
    set_boxes: WriteSignal<Vec<WorldBox>>,
    actively_moving_boxes: ReadSignal<ActivelyMovingBoxes>,
    actively_moving_boxes_set: WriteSignal<ActivelyMovingBoxes>,
) {
    let mouse = mouse_pos.get_untracked();
    let mut active_handles = Vec::new();

    // Phase 1 – set velocities for all active boxes
    for box_id in actively_moving_boxes.get_untracked().box_ids.iter() {
        let boxes_snapshot = boxes.get_untracked();
        let box_item = if let Some(item) = boxes_snapshot.iter().find(|b| &b.id == box_id) {
            item
        } else {
            continue;
        };

        let pos = box_item.position.get_untracked();
        let ctx = FigureOutDrag {
            box_x_middle: pos.0 + (box_item.width as f32) / 2.0,
            box_y_middle: pos.1 + (box_item.height as f32) / 2.0,
            mouse_x: mouse.0,
            mouse_y: mouse.1,
            old_velocity: Vec2::new(0.0, 0.0),
        };

        let desired_velocity = match box_item.animation_state.get_untracked() {
            AnimationState::ActivelyDragged => {
                let current_vel = rapier_ctx.with_untracked(|ctx| {
                    let rb = ctx.rigid_bodies.read().unwrap();
                    rb.get(box_item.rigid_body_handle)
                        .map(|body| body.linvel())
                        .unwrap_or(Vec2::new(0.0, 0.0))
                });
                let ctx_with_vel = FigureOutDrag {
                    old_velocity: current_vel,
                    ..ctx
                };
                figure_out_new_drag_velocity(ctx_with_vel)
            }
            AnimationState::TouchingMouse => crawl_to_a_stop(ctx),
            _ => continue,
        };

        // Apply velocity to rigid body
        rapier_ctx.update(|ctx| {
            let mut rb = ctx.rigid_bodies.write().unwrap();
            if let Some(body) = rb.get_mut(box_item.rigid_body_handle) {
                body.set_linvel(desired_velocity, true); // <-- direct
            }
        });

        active_handles.push((*box_id, box_item.rigid_body_handle));
    }

    // Phase 2 – step physics and read positions
    let mut new_positions = Vec::new();
    rapier_ctx.update(|ctx| {
        let mut rigid_bodies = ctx.rigid_bodies.write().unwrap();
        let mut colliders = ctx.colliders.write().unwrap();
        let mut pipeline = ctx.pipeline.write().unwrap();
        let params = ctx.params.read().unwrap();
        let mut islands = ctx.islands.write().unwrap();
        let mut broad_phase = ctx.broad_phase.write().unwrap();
        let mut narrow_phase = ctx.narrow_phase.write().unwrap();
        let mut impulse_joints = ctx.impulse_joints.write().unwrap();
        let mut multibody_joints = ctx.multibody_joints.write().unwrap();
        let mut ccd_solver = ctx.ccd_solver.write().unwrap();

        let gravity = Vector::new(0.0, 0.0 /*-9.81*/); // or your desired gravity

        pipeline.step(
            gravity,
            &params,
            &mut islands,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_bodies,
            &mut colliders,
            &mut impulse_joints,
            &mut multibody_joints,
            &mut ccd_solver,
            &(), // no custom hooks
            &(), // no event handler
        );

        // Drop read locks so we can read from rigid_bodies later
        drop(pipeline);
        drop(params);

        // Read new positions
        for (box_id, handle) in &active_handles {
            if let Some(body) = rigid_bodies.get(*handle) {
                let translation = body.translation();
                // find box dimensions from boxes signal
                let box_item = boxes
                    .get_untracked()
                    .iter()
                    .find(|b| b.id == *box_id)
                    .cloned();
                if let Some(box_item) = box_item {
                    let top_left_x = translation.x - (box_item.width as f32) / 2.0;
                    let top_left_y = translation.y - (box_item.height as f32) / 2.0;
                    new_positions.push((*box_id, (top_left_x, top_left_y)));
                }
            }
        }
    });

    // Update UI position signals
    set_boxes.update(|boxes_vec| {
        for (id, new_pos) in new_positions {
            if let Some(box_item) = boxes_vec.iter_mut().find(|b| b.id == id) {
                box_item.position.set(new_pos);
            }
        }
    });
}

fn desired_drag_velocity(mouse_pos: (f32, f32), item_pos: (f32, f32)) -> Vec2 {
    let dx = mouse_pos.0 - item_pos.0;
    let dy = mouse_pos.1 - item_pos.1;

    let max_speed = 600.0; // pixels per second
    let speed_factor = 300.0; // per second

    let mut desired = Vec2::new(dx * speed_factor, dy * speed_factor);
    if desired.length() > max_speed {
        desired = desired.normalize() * max_speed;
    }
    desired
}

fn crawl_to_a_stop(ctx: FigureOutDrag) -> Vec2 {
    let mut vel = ctx.old_velocity * 0.7;
    let dx = ctx.box_x_middle - ctx.mouse_x;
    let dy = ctx.box_y_middle - ctx.mouse_y;
    let distance = (dx * dx + dy * dy).sqrt();

    const DEAD_ZONE: f32 = 3.0;
    if distance < DEAD_ZONE {
        return Vec2::new(0.0, 0.0);
    }

    let min_speed = distance * 0.2;
    let new_vel = figure_out_new_drag_velocity(ctx);
    let speed = new_vel.length();
    if speed < min_speed {
        let dir = new_vel.try_normalize().unwrap_or(Vec2::new(0.0, 0.0));
        dir * min_speed
    } else {
        new_vel
    }
}

struct IsBoxTouchingMouseCtx {
    mouse_x: f32,
    mouse_y: f32,
    box_x: f32,
    box_y: f32,
    box_height: f32,
    box_width: f32,
}

fn is_box_touching_mouse(ctx: IsBoxTouchingMouseCtx) -> bool {
    let within_x = ctx.mouse_x >= ctx.box_x && ctx.mouse_x <= ctx.box_x + ctx.box_width;
    let within_y = ctx.mouse_y >= ctx.box_y && ctx.mouse_y <= ctx.box_y + ctx.box_height;
    within_x && within_y
}

struct FigureOutDrag {
    box_x_middle: f32,
    box_y_middle: f32,
    mouse_x: f32,
    mouse_y: f32,
    old_velocity: Vec2,
}

fn figure_out_new_drag_velocity(ctx: FigureOutDrag) -> Vec2 {
    let new_velocity = {
        let old = ctx.old_velocity;
        let desired = desired_drag_velocity(
            (ctx.mouse_x, ctx.mouse_y),
            (ctx.box_x_middle, ctx.box_y_middle),
        );

        let new_x = (old.x + desired.x) / 2.0;
        let new_y = (old.y + desired.y) / 2.0;
        Vec2::new(new_x, new_y)
    };
    new_velocity
}
