use rapier2d::geometry::ColliderBuilder;
use rapier2d::math::{Vec2, Vector};
use rapier2d::prelude::ColliderSet;
use std::cell::RefCell;
use std::rc::Rc;
use stylist::style;

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

#[component]
pub fn FunDragTestContainer() -> impl IntoView {
    let colliders = RwSignal::new(ColliderSet::new());

    let container_size = RwSignal::new(ContainerSize {
        height: 800,
        width: 1200,
    });

    let box_settings = RwSignal::new(BoxSettings {
        height: 100,
        width: 100,
    });

    let (boxes, boxes_set) = signal(Vec::new());

    let (mouse_position, mouse_position_set) = signal((0.0, 0.0));

    let (actively_moving_boxes, actively_moving_boxes_set) = signal(ActivelyMovingBoxes {
        box_ids: Vec::new(),
    });

    Effect::new(move |_| {
        // Self‑scheduling animation loop using requestAnimationFrame
        let cb: Rc<RefCell<Option<std::boxed::Box<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb_clone.borrow_mut() = Some(std::boxed::Box::new(move || {
            update_world(
                colliders,
                container_size,
                mouse_position,
                boxes,
                boxes_set,
                actively_moving_boxes,
                actively_moving_boxes_set,
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
            colliders=colliders
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
        />
    }
}

#[component]
fn MenuComponent(
    colliders: RwSignal<ColliderSet>,
    container_size: RwSignal<ContainerSize>,
    box_settings: RwSignal<BoxSettings>,
    boxes: WriteSignal<Vec<WorldBox>>,
) -> impl IntoView {
    let add_box = move || {
        let c_size = container_size.get();
        let b_settings = box_settings.get();

        let mut position = get_random_position_within_constraints(&c_size, &b_settings);

        colliders.update(|colliders| {
            for _ in 0..20 {
                if !check_if_colliding_with_another_box(colliders, position, &b_settings) {
                    break;
                }
                position = get_random_position_within_constraints(&c_size, &b_settings);
            }

            let half_width = b_settings.width as f32 / 2.0;
            let half_height = b_settings.height as f32 / 2.0;
            let center_x = position.0 as f32 + half_width;
            let center_y = position.1 as f32 + half_height;

            let collider = ColliderBuilder::cuboid(half_width, half_height)
                .translation(Vector::new(center_x, center_y))
                .build();

            let collider_handle = colliders.insert(collider);
            let id = collider_handle.into_raw_parts().0;

            boxes.update(|boxes| {
                boxes.push(WorldBox {
                    id,
                    width: b_settings.width,
                    height: b_settings.height,
                    position: RwSignal::new((position.0 as f32, position.1 as f32)),
                    animation_state: RwSignal::new(AnimationState::Still),
                    velocity: RwSignal::new(Vec2::new(0.0, 0.0)),
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
) -> impl IntoView {
    let update_mouse_position = move |pos: (f32, f32)| {
        mouse_pos.set(pos);
    };

    let (world_item_style, _) = signal(get_style!(style_for_world_item!()));

    let (box_being_dragged, box_being_dragged_set) = signal(None);

    view! {
        <div
            style:position="relative"
            style:height=move || format!("{}px", container_size.get().height)
            style:width=move || format!("{}px", container_size.get().width)
            style:border="1px black solid"
            on:mousemove=move |e| update_mouse_position((e.offset_x() as f32, e.offset_y() as f32))
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
        </div>
    }
}
