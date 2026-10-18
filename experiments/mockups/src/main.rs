//! Mockups of the design of the window: a page served by trunk, or a window
//! on the desktop. `make mockups` serves them.

use mockups::gallery::Gallery;

/// The id of the canvas of `index.html`.
#[cfg(target_arch = "wasm32")]
const CANVAS: &str = "mockups";

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    eframe::run_native(
        "obusina mockups",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(Gallery::new("")))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Info).ok();
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(CANVAS))
            .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("index.html has the canvas");
        let started = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|context| {
                    let hash = context.integration_info.web_info.location.hash.clone();
                    Ok(Box::new(Gallery::new(&hash)))
                }),
            )
            .await;
        if let Err(error) = started {
            log::error!("the mockups do not start: {error:?}");
        }
    });
}
