use rapier2d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier2d::geometry::ColliderBuilder;
use rapier2d::math::Vector;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use stylist::style;
use wasm_bindgen::JsCast; // <-- add this at the top of your file

use leptos::prelude::*;

use leptos::logging::log;

use crate::internal_calculations::*;

#[derive(Clone)]
pub struct ContainerSize {
    pub height: u32,
    pub width: u32,
}

#[derive(Clone)]
pub struct BoxSettings {
    pub height: u32,
    pub width: u32,
}

#[derive(Clone)]
pub struct ImmovableObjectSettings {
    pub width: u16,
    pub height: u16,
}

impl ImmovableObjectSettings {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

fn create_immovable_object(
    container_size: RwSignal<ContainerSize>,
    rapier_ctx: ArcRwSignal<RapierContext>,
) -> RwSignal<ImmovableObjectSettings> {
    let immovable_obj_settings = RwSignal::new(ImmovableObjectSettings::new(300u16, 500u16));

    // Store the rigid body handle so we can update its position on resize
    let immovable_body_handle = RwSignal::new(None::<RigidBodyHandle>);

    Effect::new(move |_| {
        let c_size = container_size.get();
        let imm = immovable_obj_settings.get();

        let imm_width = imm.width as f32;
        let imm_height = imm.height as f32;
        let x = (c_size.width as f32 - imm_width) / 2.0;
        let y = c_size.height as f32 - imm_height;
        let translation = Vector::new(x + imm_width / 2.0, y + imm_height / 2.0);

        rapier_ctx.update(|ctx| {
            let mut rigid_bodies = ctx.rigid_bodies.write().unwrap();
            let mut colliders = ctx.colliders.write().unwrap();

            if let Some(body_handle) = immovable_body_handle.get_untracked() {
                if let Some(body) = rigid_bodies.get_mut(body_handle) {
                    body.set_translation(translation, true);
                }
            } else {
                let body_handle =
                    rigid_bodies.insert(RigidBodyBuilder::fixed().translation(translation).build());
                let collider = ColliderBuilder::cuboid(imm_width / 2.0, imm_height / 2.0).build();
                colliders.insert_with_parent(collider, body_handle, &mut rigid_bodies);
                immovable_body_handle.set(Some(body_handle));
            }
        });
    });

    immovable_obj_settings
}

#[component]
pub fn UltimateParent() -> impl IntoView {
    let container_size = RwSignal::new(ContainerSize {
        height: 800,
        width: 1200,
    });

    let rapier_ctx = ArcRwSignal::new(RapierContext::new());
    let immovable_obj_settings = create_immovable_object(container_size, rapier_ctx.clone());

    let box_settings = RwSignal::new(BoxSettings {
        height: 100,
        width: 100,
    });

    let (boxes, boxes_set) = signal(Vec::new());

    let (mouse_position, mouse_position_set) = signal((0.0, 0.0));

    let (actively_moving_boxes, actively_moving_boxes_set) = signal(ActivelyMovingBoxes {
        box_ids: Vec::new(),
    });

    let rapier_ctx_for_effect = rapier_ctx.clone();
    Effect::new(move |_| {
        let rapier_ctx = rapier_ctx_for_effect.clone();
        let running = Arc::new(AtomicBool::new(true));
        let running_cleanup = running.clone();
        on_cleanup(move || running_cleanup.store(false, Ordering::Relaxed));
        let cb: Rc<RefCell<Option<Box<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let cb_for_frame = cb.clone();
        let cb_request = cb.clone();
        *cb.borrow_mut() = Some(Box::new(move || {
            if !running.load(Ordering::Relaxed) {
                return;
            }
            update_world(
                rapier_ctx.clone(),
                mouse_position,
                boxes,
                boxes_set,
                actively_moving_boxes,
                actively_moving_boxes_set,
            );
            let cb_weak = Rc::downgrade(&cb_for_frame);
            request_animation_frame(move || {
                if let Some(cb) = cb_weak.upgrade() {
                    if let Some(callback) = cb.borrow_mut().as_mut() {
                        callback();
                    }
                }
            });
        }));
        request_animation_frame(move || {
            if let Some(callback) = cb_request.borrow_mut().as_mut() {
                callback();
            }
        });
    });

    let rapier_ctx_for_menu = rapier_ctx.clone();
    view! {
        <MenuComponent
            rapier_ctx=rapier_ctx_for_menu
            container_size=container_size
            box_settings=box_settings
            boxes=boxes_set
        />
        <WorldComponent
            container_size=container_size
            world_boxes_set=boxes_set
            world_boxes=boxes
            actively_moving_boxes=actively_moving_boxes_set
            mouse_pos=mouse_position_set
            immovable_object_settings=immovable_obj_settings
        />
    }
}

#[component]
fn MenuComponent(
    rapier_ctx: ArcRwSignal<RapierContext>,
    container_size: RwSignal<ContainerSize>,
    box_settings: RwSignal<BoxSettings>,
    boxes: WriteSignal<Vec<WorldBox>>,
) -> impl IntoView {
    let add_box = move || {
        let c_size = container_size.get();
        let b_settings = box_settings.get();

        let mut position = get_random_position_within_constraints(&c_size, &b_settings);

        // Spawn collision check
        rapier_ctx.update(|ctx| {
            let colliders = ctx.colliders.read().unwrap();
            for _ in 0..20 {
                if !check_if_colliding_with_another_box(&*colliders, position, &b_settings) {
                    break;
                }
                position = get_random_position_within_constraints(&c_size, &b_settings);
            }
        });

        let half_width = b_settings.width as f32 / 2.0;
        let half_height = b_settings.height as f32 / 2.0;
        let center_x = position.0 as f32 + half_width;
        let center_y = position.1 as f32 + half_height;
        let translation = Vector::new(center_x, center_y);

        // Create dynamic rigid body and attach collider
        rapier_ctx.update(|ctx| {
            let mut rigid_bodies = ctx.rigid_bodies.write().unwrap();
            let mut colliders = ctx.colliders.write().unwrap();

            let rb_handle =
                rigid_bodies.insert(RigidBodyBuilder::dynamic().translation(translation).build());

            let collider = ColliderBuilder::cuboid(half_width, half_height).build();
            let collider_handle =
                colliders.insert_with_parent(collider, rb_handle, &mut rigid_bodies);
            let id = collider_handle.into_raw_parts().0;

            boxes.update(|boxes| {
                boxes.push(WorldBox {
                    id,
                    width: b_settings.width,
                    height: b_settings.height,
                    rigid_body_handle: rb_handle,
                    position: RwSignal::new((position.0 as f32, position.1 as f32)),
                    animation_state: RwSignal::new(AnimationState::Still),
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
    world_boxes_set: WriteSignal<Vec<WorldBox>>,
    world_boxes: ReadSignal<Vec<WorldBox>>,
    actively_moving_boxes: WriteSignal<ActivelyMovingBoxes>,
    mouse_pos: WriteSignal<(f32, f32)>,
    immovable_object_settings: RwSignal<ImmovableObjectSettings>,
) -> impl IntoView {
    let update_mouse_position = move |e: web_sys::MouseEvent| {
        let container = e
            .current_target()
            .expect("current_target should exist")
            .dyn_into::<web_sys::Element>() // dynamic cast
            .expect("target should be an Element");

        let rect = container.get_bounding_client_rect();
        let x = e.client_x() as f32 - rect.left() as f32;
        let y = e.client_y() as f32 - rect.top() as f32;
        mouse_pos.set((x, y));
    };

    let (world_item_style, _) = signal(get_style!(style_for_world_item!()));

    let (box_being_dragged, box_being_dragged_set) = signal(None);

    view! {
        <div
            style:position="relative"
            style:height=move || format!("{}px", container_size.get().height)
            style:width=move || format!("{}px", container_size.get().width)
            style:border="1px black solid"
            on:mousemove=move |e| update_mouse_position(e)
            on:mouseup=move |_| {
                if let Some(dragged_id) = box_being_dragged.get() {
                    actively_moving_boxes.update(|boxes| {
                        boxes.box_ids.retain(|id| *id != dragged_id);
                    });
                }
                world_boxes_set.update(|boxes| {
                    if let Some(world_box) = boxes.iter_mut().find(|b| Some(b.id) == box_being_dragged.get()) {
                        world_box.animation_state.set(AnimationState::Still);
                    }
                });
                box_being_dragged_set.set(None);
            }
        >
            {for_leptos!(world_boxes, world_box =>
                <div
                    class=format!("box box-{} {}", world_box.id, world_item_style.get())
                    style:position="absolute"
                    style:left=move || format!("{:.1}px", world_box.position.get().0)
                    style:top=move || format!("{:.1}px", world_box.position.get().1)
                    style:width=move || format!("{}px", world_box.width)
                    style:height=move || format!("{}px", world_box.height)
                    style:background-color="lightblue"
                    style:border="1px solid black"
                    on:mousedown=move |_| {
                        //update internal id of which box is being dragged so it knows which id to remove on mouseup
                        box_being_dragged_set.set(Some(world_box.id));
                        //update the list of actively moving boxes so the thing that manages the actual movement is aware of the drag
                        actively_moving_boxes.update(|boxes| {
                            boxes.box_ids.push(world_box.id);
                        });
                        //update the enum for the correct box in the list of all the WorldBoxes
                        world_boxes_set.update(|boxes| {
                            if let Some(world_box) = boxes.iter_mut().find(|b| b.id == world_box.id) {
                                world_box.animation_state.set(AnimationState::ActivelyDragged);
                            }
                        });
                    }
                />
            )}
            <ImmovableObject immovable_obj_settings=immovable_object_settings />
        </div>
    }
}

#[component]
fn ImmovableObject(immovable_obj_settings: RwSignal<ImmovableObjectSettings>) -> impl IntoView {
    view! {
        <div
            style:position="absolute"
            style:bottom="0"
            style:left="50%"
            style:transform="translateX(-50%)"
            style:width=move || format!("{}px", immovable_obj_settings.get().width)
            style:height=move || format!("{}px", immovable_obj_settings.get().height)
            style:border="1px solid black"
            style:border-bottom="none"
        >
        </div>
    }
}
