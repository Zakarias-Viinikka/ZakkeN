// for_leptos! - shorthand for leptos's <For> component.
//
// <For> needs three things: each (what to loop over), key (how to tell
// items apart), and children (what to render per item). This macro hides
// those three behind a shape that reads like a normal for loop.
//
// Usage:
//
//   {for_leptos!(list, item =>
//       <div>{item.text.clone()}</div>
//   )}
//
//   list   the signal. Pass the signal itself, not list.get(). The macro
//          adds the .get() inside a closure so it stays reactive.
//   item   the name for one element inside the body. Anything you want.
//   body   the view for one element, straight after "=>". No inner
//          view! {} needed - the macro wraps it.
//
// The struct being looped over must:
//   - derive Clone
//   - have an "id" field. The macro uses .id as the key. Ids must be
//     unique and stable - an id, not the content itself, or leptos
//     loses track when content changes.
#[macro_export]
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

//--
// for the db
//--

macro_rules! beg_js_to_work_the_worker {
    ($cmd:expr) => {{
        let arr = js_sys::Array::new();
        arr.push(&JsValue::from_str($cmd));
        beg_js(arr).await
    }};
    ($cmd:expr, $payload:expr) => {{
        let arr = js_sys::Array::new();
        arr.push(&JsValue::from_str($cmd));
        let bytes: Vec<u8> = $payload;
        let uint8 = js_sys::Uint8Array::from(&bytes[..]);
        arr.push(&uint8);
        beg_js(arr).await
    }};
}
