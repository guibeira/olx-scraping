use serde::{Deserialize, Serialize};
use gloo_console::log;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    
    log!("Iniciando o app...");
    let greet_input_ref = use_node_ref();
    let path_input_ref = use_node_ref();

    let name = use_state(|| String::new());
    let folder = use_state(|| String::new());

    let greet_msg = use_state(|| String::new());
    {
        let greet_msg = greet_msg.clone();
        let name = name.clone();
        let folder = folder.clone();
        let name2 = name.clone();
        let folder2 = folder.clone();

        use_effect_with(
            [name2, folder2],
            move |_| {
                spawn_local(async move {
                    
                    if name.is_empty() {
                        return;
                    }

                    log!("Invoking the greet command...");
                    let args = to_value(&GreetArgs { name: &*name}).unwrap();
                    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                    let new_msg = invoke("greet", args).await.as_string();
                    if new_msg.is_none() {
                        log!("Error invoking the greet command");
                        return;
                    }
                    let new_msg = new_msg.unwrap();

                    greet_msg.set(new_msg);
                });

                || {}
            },
        );
    }

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        let path_input_ref = path_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(
                greet_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );

            folder.set(
                path_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };

    html! {
        <main class="container">
            <div class="row">
                <p>{"Teje bem vindo"}</p>
            </div>
            <form class="row" onsubmit={greet}>
               <div class="input-container"> 
                    <input id="greet-input" ref={greet_input_ref} value={"https://www.olx.com.br/autos-e-pecas/carros-vans-e-utilitarios/estado-pr?pe=50000&ps=10000&re=74&rs=67"}placeholder="Entre com a url já com os filtros..." />
                <button type="submit">{"Socá a porva"}</button>
                </div>
            </form>

            <p><b>{ &*greet_msg }</b></p>
        </main>
    }
}
