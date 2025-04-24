use std::sync::Arc;

use dioxus::{html::{FileEngine, HasFileData}, prelude::*};
use reqwest::multipart::{self, Part};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
// const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/icon?family=Material+Icons" }
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn MainLayout() -> Element {
    rsx! {
        div {
            class: "text-white flex flex-col h-screen",
            style: "background-color: #1b1b1b;",
            
            Navbar {  }
            div {
                class: "flex-grow",
                // This is where the routed content will be rendered
                Sidebar {  }
                div {
                    class: "h-screen p-2 pt-17 ml-64",
                    div {
                        class: "px-6 py-5 rounded-3xl bg-neutral-950 h-full",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct File {
    id: String,
    name: String,
    size: u64,
    last_modified: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Folder {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Entries {
    folders: Vec<Folder>,
    files: Vec<File>
}

/// Home page
#[component]
fn Home() -> Element {
    // get files from the server
    let mut files = use_resource(|| async move {
        reqwest::Client::new()
            .post("http://localhost:8000/api/v1/files")
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap()
            .json::<Entries>()
            .await
    });

    rsx! {
        div {
            class: "",
            h1 { class: "text-3xl", "My Cloud Drive" }
            // a list of files (full width, separated by lines)
            div {
                class: "mt-4 flex flex-col divide-y-1 divide-neutral-700",
                // for each file, create a div with a description icon and the file name
                // use the future from above to get the files
                match &*files.read_unchecked() {
                    Some(Ok(response)) => rsx! {
                        for file in &response.files {
                            div {
                                class: "flex px-2 p-1 items-center",
                                div {
                                    class: "w-4 h-4 flex items-center justify-center text-neutral-400",
                                    i { class: "material-icons", "description" }
                                }
                                div {
                                    class: "ml-2",
                                    h2 { class: "", "{file.name}" }
                                }
                            }
                        }
                    },
                    Some(Err(_)) => rsx! {
                        div { "Loading dogs failed" }
                    },
                    None => rsx! {
                        div { "Loading dogs..." }
                    },
                }

                // for file in ["file1.txt", "file2.txt", "file3.txt"] {
                //     div {
                //         class: "flex px-2 p-1 items-center",
                //         div {
                //             class: "w-4 h-4 flex items-center justify-center text-neutral-400",
                //             i { class: "material-icons", "description" }
                //         }
                //         div {
                //             class: "ml-2",
                //             h2 { class: "", "{file}" }
                //         }
                //     }
                // }
            }
            div {
                h1 { class: "text-3xl mt-4", "Upload" }
                FileUpload {  }
            }
        }
    }
}

struct UploadedFile {
    name: String,
    contents: String,
}

#[component]
fn FileUpload() -> Element {
    let mut enable_directory_upload = use_signal(|| false);
    let mut files_uploaded = use_signal(|| Vec::new() as Vec<UploadedFile>);
    let mut hovered = use_signal(|| false);

    let read_files = move |file_engine: Arc<dyn FileEngine>| async move {
        let files = file_engine.files();
        for file_name in &files {
            if let Some(contents) = file_engine.read_file_to_string(file_name).await {
                files_uploaded.write().push(UploadedFile {
                    name: file_name.clone(),
                    contents,
                });
            }
        }
    };

    let load_files = move |evt: FormEvent| async move {
        if let Some(file_engine) = evt.files() {
            read_files(file_engine).await;
        }
    };

    let upload_files = move || async move {
        let mut multipart = reqwest::multipart::Form::new();
        
        for file in files_uploaded.read().iter() {
            let part = Part::text(file.contents.clone())
                .file_name(file.name.clone())
                .mime_str("text/plain").unwrap();
            multipart = multipart.part(file.name.clone(), part);
        };

        let client = reqwest::Client::new();
        client.post("http://localhost:8000/api/v1/upload")
            .multipart(multipart)
            .send()
            .await
            .unwrap();
    };

    rsx! {
        h1 { "File Upload Example" }
        p { "Drop a .txt, .rs, or .js file here to read it" }
        button { onclick: move |_| files_uploaded.write().clear(), "Clear files" }

        div {
            label { r#for: "directory-upload", "Enable directory upload" }
            input {
                r#type: "checkbox",
                id: "directory-upload",
                checked: enable_directory_upload,
                oninput: move |evt| enable_directory_upload.set(evt.checked()),
            }
        }

        div {
            label { r#for: "textreader", "Upload text/rust files and read them" }
            input {
                r#type: "file",
                accept: ".txt,.rs,.js",
                multiple: true,
                name: "textreader",
                directory: enable_directory_upload,
                onchange: load_files,
            }
        }

        div {
            id: "drop-zone",
            background_color: if hovered() { "lightblue" } else { "lightgray" },
            ondragover: move |evt| {
                evt.prevent_default();
                hovered.set(true)
            },
            ondragleave: move |_| hovered.set(false),
            ondrop: move |evt| async move {
                evt.prevent_default();
                hovered.set(false);
                if let Some(file_engine) = evt.files() {
                    read_files(file_engine).await;
                }
            },
            "Drop files here"
        }

        ul {
            for file in files_uploaded.read().iter().rev() {
                li {
                    span { "{file.name}" }
                }
            }
        }

        button {
            onclick: move |_| upload_files(),
            "Upload files"
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            class: "w-full fixed top-0",
            h1 { class: "text-2xl", "My Dioxus App" }

        }
    }
}

#[component]
fn Sidebar() -> Element {
    let path: Route = use_route();
    
    rsx! {
        div {
            class: "w-64 pt-17 h-full fixed",
            nav {
                class: "flex flex-col gap-2 h-full p-4",
                Link {
                    to: Route::Home {},
                    class: "flex items-center px-4 p-2 rounded-lg hover:bg-neutral-700",
                    active_class: "bg-blue-500/50 hover:bg-blue-500/30",
                    "Home"
                }
                Link {
                    to: Route::Home {},
                    class: "flex items-center px-4 p-2 rounded-lg hover:bg-neutral-700",
                    active_class: "bg-blue-500/50 hover:bg-blue-500/30",
                    "Recent"
                }
                Link {
                    to: Route::Home {},
                    class: "flex items-center px-4 p-2 rounded-lg hover:bg-neutral-700",
                    active_class: "bg-blue-500/50 hover:bg-blue-500/30",
                    "Shared"
                }
                Link {
                    to: Route::Home {},
                    class: "flex items-center px-4 p-2 rounded-lg hover:bg-neutral-700",
                    active_class: "bg-blue-500/50 hover:bg-blue-500/30",
                    "Starred"
                }
                Link {
                    to: Route::Home {},
                    class: "flex items-center px-4 p-2 rounded-lg hover:bg-neutral-700",
                    active_class: "bg-blue-500/50 hover:bg-blue-500/30",
                    "Trash"
                }
            }
        }
    }
}
