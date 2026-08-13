use leptos::prelude::*;

#[component]
pub fn JsStuff() -> impl IntoView {
    view! {
        /*
         * the javascript files are in root/finale/...
         *
         * reminder that they need to be linked to in index.html
         * or some other solution.
         */
         <script>
             "
                console.log('hello');
             "
         </script>
    }
}
