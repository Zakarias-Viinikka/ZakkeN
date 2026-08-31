use rapier2d::prelude::*;
use std::time::Duration;

use leptos::prelude::*;

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
        World {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
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

    Effect::new(move |_| {
        set_interval(move || update_world(world), Duration::from_millis(50));
    });
    view! {
        <ManualInput
            world=world
            container_size=container_size
            box_settings=box_settings
            boxes=boxes_set
        />
        <WorldComponent
            world=world
            container_size=container_size
            boxes=boxes
        />
    }
}

#[component]
fn ManualInput(
    world: RwSignal<World>,
    container_size: RwSignal<ContainerSize>,
    box_settings: RwSignal<BoxSettings>,
    boxes: WriteSignal<Vec<Box>>,
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

            let collider = ColliderBuilder::cuboid(half_width, half_height).build();

            let body_handle = w.bodies.insert(rigid_body);

            w.colliders
                .insert_with_parent(collider, body_handle, &mut w.bodies);

            boxes.update(|boxes| {
                boxes.push(Box {
                    id: body_handle.into_raw_parts().0,
                    width: b_settings.width,
                    height: b_settings.height,
                    position,
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
    let r = js_sys::Math::random(); // 0.0..1.0
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
        let center = collider.translation();
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
struct Box {
    id: u32,
    width: u32,
    height: u32,
    position: (u32, u32),
}

#[component]
fn WorldComponent(
    world: RwSignal<World>,
    container_size: RwSignal<ContainerSize>,
    boxes: ReadSignal<Vec<Box>>,
) -> impl IntoView {
    view! {
        <div
            style:position="relative"
            style:height=move || format!("{}px", container_size.get().height)
            style:width=move || format!("{}px", container_size.get().width)
            style:border="1px black solid"
        >
        {for_leptos!(boxes, box_item =>
            <div
                style:position="absolute"
                style:left=move || format!("{}px", box_item.position.0)
                style:top=move || format!("{}px", box_item.position.1)
                style:width=move || format!("{}px", box_item.width)
                style:height=move || format!("{}px", box_item.height)
                style:background-color="lightblue"
                style:border="1px solid black"
            />
        )}
        </div>
    }
}

fn update_world(world: RwSignal<World>) {}
