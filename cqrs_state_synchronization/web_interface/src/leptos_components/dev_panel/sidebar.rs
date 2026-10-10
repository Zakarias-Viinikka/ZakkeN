use leptos::{logging::log, prelude::*};
use stylist::style;

const SIDEBAR_TOGGLE_BUTTON_WIDTH: i32 = 32;
const SIDEBAR_PADDING: i32 = 20;

macro_rules! get_class {
    ($style_var:ident) => {
        $style_var.get_class_name().to_string()
    };
}
macro_rules! style_for_sidebar {
    () => {
        style!(
            r#"
            /* Layout and positioning */
            position: fixed;
            left: 0;
            top: 0;
            height: 100%;
            width: 240px;
            box-sizing: border-box;
            display: flex;
            flex-direction: column;

            /* Colors and typography */
            background: linear-gradient(180deg, rgb(52, 73, 94) 0%, #2c3e50 100%);
            color: #ecf0f1;
            font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;

            /* Spacing and borders */
            padding: ${p}px;
            padding-top: 10px;
            border-right: 1px solid #1a252f;

            /* Shadow and overflow */
            box-shadow: 2px 0 5px rgba(0, 0, 0, 0.2);
            overflow-y: auto;

            /* Animation */
            transition: transform 0.3s ease;
            "#,
            p = SIDEBAR_PADDING
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}
macro_rules! style_for_close_open_button {
    () => {
        style!(
            r#"
            /* align to the right */
            align-self: flex-end;
            margin-right: -${p}px;

            /* Circular icon button – no background, hover effect */
            width: ${w}px;
            height: 32px;
            padding: 0;
            border: none;
            border-radius: 50%;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            background: transparent;
            color: #ecf0f1;
            cursor: pointer;
            line-height: 0;
            box-sizing: border-box;
            transition: background 0.2s;

            /* Hover and focus effects */
            &:hover {
                background: rgba(255, 255, 255, 0.15);
            }
            &:focus {
                outline: none;
                box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.5);
            }
            "#,
            w = SIDEBAR_TOGGLE_BUTTON_WIDTH,
            p = SIDEBAR_PADDING / 2,
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}
macro_rules! style_for_content_container {
    () => {
        style!(
            /* Layout and spacing */
            padding: 20px;
            margin: 10px;

            /* Border and background */
            border: 2px solid black;

            /* Typography */
            font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;

            /* Transition for opacity */
            transition: opacity 0.3s ease;
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}
macro_rules! style_for_fade_out {
    () => {
        style!(
            opacity: 0;
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}
macro_rules! style_for_slide_effect {
    () => {
        style!(
            r#"transform: translateX(calc(-100% + ${w}px + ${p}px));"#,
            w = SIDEBAR_TOGGLE_BUTTON_WIDTH,
            p = SIDEBAR_PADDING
        )
        .map_err(|e| log!("{}", e))
        .unwrap()
    };
}

#[component]
pub fn SidebarContainer() -> impl IntoView {
    let is_sidebar_active = RwSignal::new(false);

    let sidestyle = style_for_sidebar!();
    let close_open_button = style_for_close_open_button!();
    let fade_out = style_for_fade_out!();
    let content_container = style_for_content_container!();
    let slide_effect = style_for_slide_effect!();

    let (container_without_fade, _) = signal(content_container.get_class_name().to_string());
    let (container_with_fade, _) = signal(format!(
        "{} {}",
        container_without_fade.get(),
        fade_out.get_class_name().to_string()
    ));

    let (just_sidestyle, _) = signal(sidestyle.get_class_name().to_string());
    let (sidestyle_with_slide_applied, _) = signal(format!(
        "{} {}",
        just_sidestyle.get(),
        slide_effect.get_class_name().to_string()
    ));

    view! {
        <div class=move || {
            if is_sidebar_active.get() {
                just_sidestyle.get()
            } else {
                sidestyle_with_slide_applied.get()
            }
        }/*get_class!(sidestyle)*/>
            <button
                on:click=move |_| {
                    is_sidebar_active.update(|active| *active = !*active);
                }
                class=get_class!(close_open_button)>
                { move || {
                    if is_sidebar_active.get() {
                        view! {
                            <span inner_html=RIGHT_ARROW_IMG></span>
                        }
                    } else {
                        view! {
                            <span inner_html=LEFT_ARROW_IMG></span>
                        }
                    }
                }}
            </button>
            <div class=move || {
                if is_sidebar_active.get() {
                    container_without_fade.get()
                } else {
                    container_with_fade.get()
                }
            }>
                    <SidebarContent />
                </div>
        </div>
    }
}

#[component]
fn SidebarContent() -> impl IntoView {
    view! {
        "tst"
        <br/>
        "bst"
    }
}

// i modified them tho:
// https://iconmonstr.com/caret-right-circle-filled-svg/
const RIGHT_ARROW_IMG: &str = r#"<svg width="24" height="24" clip-rule="evenodd" fill-rule="evenodd" stroke-linejoin="round" stroke-miterlimit="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="m2.009 12.002c0-5.517 4.48-9.997 9.998-9.997s9.998 4.48 9.998 9.997c0 5.518-4.48 9.998-9.998 9.998s-9.998-4.48-9.998-9.998zm8.211-4.843c-.141-.108-.3-.157-.456-.157-.389 0-.755.306-.755.749v8.501c0 .445.367.75.755.75.157 0 .316-.05.457-.159 1.554-1.203 4.199-3.252 5.498-4.258.184-.142.29-.36.29-.592 0-.23-.107-.449-.291-.591z" fill-rule="nonzero"/></svg>"#;

// https://iconmonstr.com/caret-left-circle-filled-svg/
const LEFT_ARROW_IMG: &str = r#"<svg width="24" height="24" clip-rule="evenodd" fill-rule="evenodd" stroke-linejoin="round" stroke-miterlimit="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="m22 12.002c0-5.517-4.48-9.997-9.998-9.997-5.517 0-9.997 4.48-9.997 9.997 0 5.518 4.48 9.998 9.997 9.998 5.518 0 9.998-4.48 9.998-9.998zm-8.211-4.843c.141-.108.3-.157.456-.157.389 0 .755.306.755.749v8.501c0 .445-.367.75-.755.75-.157 0-.316-.05-.457-.159-1.554-1.203-4.199-3.252-5.498-4.258-.184-.142-.29-.36-.29-.592 0-.23.107-.449.291-.591z" fill-rule="nonzero"/></svg>"#;
