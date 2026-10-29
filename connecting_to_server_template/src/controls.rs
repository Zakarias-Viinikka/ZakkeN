use crate::client_server_coms::ComsHandleForClient;
use crate::CustomButton;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Controls(coms: ComsHandleForClient) -> impl IntoView {
    let coms_connect = coms.clone();
    let coms_wipe = coms.clone();
    view! {
        <div class="controls">
            <div class="controls-section">
                <h3 class="controls-title">"Client"</h3>
                <hr class="controls-hr"/>
                <CustomButton
                    label="Connect to server"
                    on_click=move |_| {
                        let coms = coms_connect.clone();
                        spawn_local(async move {
                            let _online = coms.is_server_online().await;
                        });
                    }
                />
                <CustomButton label="Disconnect from server"/>
            </div>
            <div class="controls-section">
                <h3 class="controls-title">"Popups"</h3>
                <hr class="controls-hr"/>
                <CustomButton
                    label="Show popup"
                    on_click=move |_| crate::cool_popup::show_popup("Hello from cool popup")
                />
                <CustomButton
                    label="Show error"
                    on_click=move |_| crate::cool_popup::show_popup_as(
                        crate::cool_popup::PopupKind::Error,
                        "Something went wrong",
                    )
                />
            </div>
            <div class="controls-section">
                <h3 class="controls-title">"Server"</h3>
                <hr class="controls-hr"/>
                <CustomButton label="Turn off server"/>
                <CustomButton label="Start server"/>
                <CustomButton
                    label="Wipe server"
                    on_click=move |_| coms_wipe.wipe_server()
                />
            </div>
        </div>
    }
}