//https://rapier.rs/docs/user_guides/rust/getting_started/
use rapier2d::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use stylist::style;

use leptos::prelude::*;

use leptos::logging::log;

struct World {
    pipeline: PhysicsPipeline,
    gravity: Vector,
    integration_parameters: IntegrationParameters,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl World {
    fn new() -> Self {
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = 1.0 / 60.0; // fixed timestep

        World {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector::new(0.0, 0.0),
            integration_parameters,
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }
}

#[derive(Clone)]
struct ContainerSize {
    height: u32,
    width: u32,
}

#[derive(Clone)]
struct BoxSettings {
    height: u32,
    width: u32,
}

#[component]
pub fn FunDragTestContainer() -> impl IntoView {
    let world = RwSignal::new(World::new());

    let container_size = RwSignal::new(ContainerSize {
        height: 800,
        width: 1200,
    });

    let box_settings = RwSignal::new(BoxSettings {
        height: 100,
        width: 100,
    });

    let (boxes, boxes_set) = signal(Vec::new());

    let (box_being_dragged, box_being_dragged_set) = signal(None);
    let (mouse_position, mouse_position_set) = signal((0.0, 0.0));

    Effect::new(move |_| {
        // Self‑scheduling animation loop using requestAnimationFrame
        let cb: Rc<RefCell<Option<std::boxed::Box<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb_clone.borrow_mut() = Some(std::boxed::Box::new(move || {
            update_world(
                world,
                container_size,
                box_being_dragged,
                mouse_position,
                boxes,
                1.0 / 60.0, // fixed timestep
            );

            let cb_weak = Rc::downgrade(&cb);
            request_animation_frame(move || {
                if let Some(cb) = cb_weak.upgrade() {
                    if let Some(callback) = cb.borrow_mut().as_mut() {
                        callback();
                    }
                }
            });
        }));

        request_animation_frame(move || {
            if let Some(callback) = cb_clone.borrow_mut().as_mut() {
                callback();
            }
        });
    });

    view! {
        <MenuComponent
            world=world
            container_size=container_size
            box_settings=box_settings
            boxes=boxes_set
        />
        <WorldComponent
            container_size=container_size
            boxes=boxes
            box_being_dragged=box_being_dragged_set
            mouse_pos=mouse_position_set
        />
    }
}

#[component]
fn MenuComponent(
    world: RwSignal<World>,
    container_size: RwSignal<ContainerSize>,
    box_settings: RwSignal<BoxSettings>,
    boxes: WriteSignal<Vec<WorldBox>>,
) -> impl IntoView {
    let add_box = move || {
        let c_size = container_size.get();
        let b_settings = box_settings.get();

        let mut position = get_random_position_within_constraints(&c_size, &b_settings);

        world.update(|w| {
            for _ in 0..20 {
                if !check_if_colliding_with_another_box(w, position, &b_settings) {
                    break;
                }
                position = get_random_position_within_constraints(&c_size, &b_settings);
            }

            let half_width = b_settings.width as f32 / 2.0;
            let half_height = b_settings.height as f32 / 2.0;
            let center_x = position.0 as f32 + half_width;
            let center_y = position.1 as f32 + half_height;

            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(Vector::new(center_x, center_y))
                .build();

            // Normal density for stable behavior
            let collider = ColliderBuilder::cuboid(half_width, half_height).build();

            let body_handle = w.bodies.insert(rigid_body);

            w.colliders
                .insert_with_parent(collider, body_handle, &mut w.bodies);

            boxes.update(|boxes| {
                boxes.push(WorldBox {
                    id: body_handle.into_raw_parts().0,
                    width: b_settings.width,
                    height: b_settings.height,
                    position: RwSignal::new((position.0 as f32, position.1 as f32)),
                });
            });
        });
    };

    view! {
        <button on:click=move |_| add_box()>
            "Add box"
        </button>
    }
}

fn random_range(min: u32, max: u32) -> u32 {
    if max <= min {
        return min;
    }
    let r = js_sys::Math::random();
    min + (r * (max - min) as f64) as u32
}

fn get_random_position_within_constraints(
    container_size: &ContainerSize,
    box_settings: &BoxSettings,
) -> (u32, u32) {
    let max_x = container_size.width.saturating_sub(box_settings.width);
    let max_y = container_size.height.saturating_sub(box_settings.height);
    (random_range(0, max_x), random_range(0, max_y))
}

fn check_if_colliding_with_another_box(
    world: &World,
    position: (u32, u32),
    box_settings: &BoxSettings,
) -> bool {
    let new_left = position.0 as f32;
    let new_top = position.1 as f32;
    let new_right = new_left + box_settings.width as f32;
    let new_bottom = new_top + box_settings.height as f32;

    for (_, collider) in world.colliders.iter() {
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

macro_rules! for_leptos {
    ($list:expr, $item:ident => $($body:tt)+) => {
        view! {
            <For
                each=move || $list.get()
                key=|$item| $item.id
                children=move |$item| view! { $($body)+ }
            />
        }
    };
}

#[derive(Clone)]
struct WorldBox {
    id: u32,
    width: u32,
    height: u32,
    position: RwSignal<(f32, f32)>, // floating point for smooth rendering
}

macro_rules! style_for_world_item {
    () => {
        style!(
            r#"
            &:hover {
            cursor: crosshair;
            }
            "#
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}

macro_rules! get_style {
    ($item:expr) => {
        $item.get_class_name().to_string()
    };
}

#[component]
fn WorldComponent(
    container_size: RwSignal<ContainerSize>,
    boxes: ReadSignal<Vec<WorldBox>>,
    box_being_dragged: WriteSignal<Option<u32>>,
    mouse_pos: WriteSignal<(f32, f32)>,
) -> impl IntoView {
    let update_mouse_position = move |pos: (f32, f32)| {
        mouse_pos.set(pos);
    };

    let (world_item_style, _) = signal(get_style!(style_for_world_item!()));

    view! {
        <div
            style:position="relative"
            style:height=move || format!("{}px", container_size.get().height)
            style:width=move || format!("{}px", container_size.get().width)
            style:border="1px black solid"
            on:mousemove=move |e| update_mouse_position((e.offset_x() as f32, e.offset_y() as f32))
            on:mouseup=move |_| {
                box_being_dragged.set(None);
            }
        >
            {for_leptos!(boxes, box_item =>
                <div
                    class=format!("box box-{} {}", box_item.id, world_item_style.get())
                    style:position="absolute"
                    style:left=move || format!("{:.1}px", box_item.position.get().0)
                    style:top=move || format!("{:.1}px", box_item.position.get().1)
                    style:width=move || format!("{}px", box_item.width)
                    style:height=move || format!("{}px", box_item.height)
                    style:background-color="lightblue"
                    style:border="1px solid black"
                    on:mousedown=move |_| {
                        box_being_dragged.set(Some(box_item.id));
                    }
                />
            )}
        </div>
    }
}

// NEW: Extracted velocity calculation method.
// You can modify this function to change how the dragged box follows the cursor.
fn calculate_drag_velocity(
    body_pos: Vector,
    mouse_pos: (f32, f32),
    current_vel: Vector,
    container_size: &ContainerSize,
) -> Vector {
    // Clamp mouse to container boundaries to avoid huge forces when cursor leaves the area
    let clamped_mx = mouse_pos.0.max(0.0).min(container_size.width as f32);
    let clamped_my = mouse_pos.1.max(0.0).min(container_size.height as f32);
    let error = Vector::new(clamped_mx - body_pos.x, clamped_my - body_pos.y);

    // Adjust stiffness and damping to change feel.
    // Higher stiffness = stronger pull toward mouse.
    // Higher damping = more resistance, slows down faster.
    let stiffness = 5.0;
    let damping = 10.0;
    let mut target_vel = error * stiffness - current_vel * damping;

    // Limit maximum speed to prevent violent flinging
    let max_speed = 200.0;
    let norm = (target_vel.x * target_vel.x + target_vel.y * target_vel.y).sqrt();
    if norm > max_speed {
        target_vel = target_vel * (max_speed / norm);
    }

    target_vel
}

fn update_world(
    world: RwSignal<World>,
    container_size: RwSignal<ContainerSize>,
    box_being_dragged: ReadSignal<Option<u32>>,
    mouse_pos: ReadSignal<(f32, f32)>,
    boxes: ReadSignal<Vec<WorldBox>>,
    dt: f32,
) {
    let (mx, my) = mouse_pos.get_untracked();
    let c_size = container_size.get_untracked();

    world.update(|w| {
        // Set variable timestep
        w.integration_parameters.dt = dt;

        // Step physics
        w.pipeline.step(
            w.gravity,
            &w.integration_parameters,
            &mut w.islands,
            &mut w.broad_phase,
            &mut w.narrow_phase,
            &mut w.bodies,
            &mut w.colliders,
            &mut w.impulse_joints,
            &mut w.multibody_joints,
            &mut w.ccd_solver,
            &(),
            &(),
        );

        // Global friction
        for (_handle, body) in w.bodies.iter_mut() {
            if body.is_dynamic() {
                let linvel = body.linvel();
                body.set_linvel(linvel * 0.98, true);
            }
        }

        // Drag using the extracted velocity calculation
        if let Some(id) = box_being_dragged.get_untracked() {
            let handle = RigidBodyHandle::from_raw_parts(id, 0);
            if let Some(body) = w.bodies.get_mut(handle) {
                let pos = body.translation();
                let current_vel = body.linvel();
                let target_vel = calculate_drag_velocity(pos, (mx, my), current_vel, &c_size);
                body.set_linvel(target_vel, true);
            }
        }

        // Clamp all boxes inside container
        boxes.with_untracked(|list| {
            for b in list {
                let handle = RigidBodyHandle::from_raw_parts(b.id, 0);
                if let Some(body) = w.bodies.get_mut(handle) {
                    let pos = body.translation();
                    let half_w = b.width as f32 / 2.0;
                    let half_h = b.height as f32 / 2.0;
                    let min_x = half_w;
                    let max_x = c_size.width as f32 - half_w;
                    let min_y = half_h;
                    let max_y = c_size.height as f32 - half_h;

                    let mut new_pos = pos;
                    let mut new_vel = body.linvel();
                    let mut clamped = false;

                    if new_pos.x < min_x {
                        new_pos.x = min_x;
                        new_vel.x = 0.0;
                        clamped = true;
                    }
                    if new_pos.x > max_x {
                        new_pos.x = max_x;
                        new_vel.x = 0.0;
                        clamped = true;
                    }
                    if new_pos.y < min_y {
                        new_pos.y = min_y;
                        new_vel.y = 0.0;
                        clamped = true;
                    }
                    if new_pos.y > max_y {
                        new_pos.y = max_y;
                        new_vel.y = 0.0;
                        clamped = true;
                    }

                    if clamped {
                        body.set_translation(new_pos, true);
                        body.set_linvel(new_vel, true);
                    }
                }
            }
        });
    });

    // Update UI positions
    world.with_untracked(|w| {
        boxes.with_untracked(|list| {
            for b in list {
                let handle = RigidBodyHandle::from_raw_parts(b.id, 0);
                if let Some(body) = w.bodies.get(handle) {
                    let pos = body.translation();
                    b.position
                        .set((pos.x - b.width as f32 / 2.0, pos.y - b.height as f32 / 2.0));
                }
            }
        });
    });
}
