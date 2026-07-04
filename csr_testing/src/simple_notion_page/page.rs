use crate::javascript_take_the_wheel;
use crate::simple_notion_page::jsval_parsing;
use crate::simple_notion_page::textarea::TextArea;

use leptos::logging::log;
use leptos::prelude::*;

#[derive(Clone)] //necessary
struct TextBlocks {
    text: RwSignal<String>,
    id: usize,
}

impl TextBlocks {
    fn new(id: usize) -> Self {
        Self {
            text: RwSignal::new(String::new()),
            id,
        }
    }
}

#[component]
pub fn SimpleNotionPage() -> impl IntoView {
    let list = RwSignal::new(Vec::new());

    //make text blocks
    list.update(|l| {
        for _ in 0..5 {
            l.push(TextBlocks::new(l.len()));
        }
    });
    //make text blocks

    //js handle
    javascript_take_the_wheel!("update_list_order", |js_value| {
        match jsval_parsing::js_value_to_usize_tuple(js_value) {
            Ok((old_index, new_index)) => {
                list.update(|v| {
                    let item = v.remove(old_index);
                    v.insert(new_index, item);
                });
            }
            Err(e) => log!("{}", e), //console.log error
        }
    });
    //js handle

    view! {
        <div class="container">

            <ul id="sortable-container">
                 <ForEnumerate
                     each=move || list.get()
                     key=|text_blocks| text_blocks.id
                     let(index, text_blocks)
                >
                    <TextArea
                        index=index
                        text=text_blocks.text
                    />
                 </ForEnumerate>
             </ul>
        </div>

        <div>
        "this is all of the textblocks combined:"
        <br/>
        <ForEnumerate
            each=move || list.get()
            key=|text_blocks| text_blocks.id
            let(_, text_blocks)
        >
           <span>
               {move ||
                   text_blocks.text.get()
               }
               <br/>
           </span>
        </ForEnumerate>
        </div>
    }
}
