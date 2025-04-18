#[cfg(feature = "server")]
mod backend;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    id: u32,
    name: String,
    checked: bool,
}

fn main() {
    #[cfg(feature = "server")]
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(backend::launch());
    #[cfg(not(feature = "server"))]
    dioxus::launch(dooT);
}

#[component]
pub fn dooT() -> Element {
    let items = use_server_future(read_items)?().unwrap().unwrap();
    let mut item_list = use_signal(Vec::<Item>::new);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            h3 { "dooT" }
            ItemInput { }
            ul {
                id: "theList",
                for item in items {
                    ItemElement { item: item.clone(), item_list }
                }
            }
        }
    }
}

#[component]
fn ItemInput() -> Element {
    let mut item_name = use_signal(String::new);
    rsx! {
        div {
            class: "header",
            input {
                type:  "text",
                id: "myInput",
                placeholder: "Item Name...",
                class: "input",
                value: item_name,
                oninput: move |event| {
                    item_name.set(event.value());
                },
                onkeydown: move |event| async move {
                    if event.code().to_string() == "Enter" {
                        add_item(item_name()).await.unwrap();
                        item_name.set(String::new());
                    }
                }
            }
        },
    }
}

#[component]
fn ItemElement(item: Item, item_list: Signal<Vec<Item>>) -> Element {
    rsx! {
        li {
            onclick: move |_event| async move {
                toggle_checked(item.id).await.unwrap();
            },
            class: if item.checked {"checked"},
            label { { item.name.clone() } },
            span {
                class: "close",
                onclick: move |_event| {
                    let item = item.clone();
                    async move {
                        delete_item(item).await.unwrap();
                    }
                },
                class: "delete-button",
                "Delete"
            }
        }
    }
}

#[server]
pub async fn add_item(item: String) -> Result<(), ServerFnError> {
    backend::DB.with(|f| {
        f.execute("INSERT INTO dooT (name, checked) VALUES(?1, 0)", [item])
            .unwrap();
    });
    Ok(())
}

#[server]
pub async fn delete_item(item: Item) -> Result<(), ServerFnError> {
    backend::DB.with(|f| {
        f.execute("DELETE FROM dooT WHERE id=?1", [item.id])
            // .unwrap() // try to use this instead and see what errors you get.
            .unwrap();
    });
    Ok(())
}

#[server]
pub async fn toggle_checked(id: u32) -> Result<(), ServerFnError> {
    backend::DB.with(|f| {
        f.execute(
            "
    UPDATE dooT
    SET checked = CASE checked
                  WHEN 0 THEN 1
                  ELSE        0
                  END
    WHERE id = ?1;
    ",
            [id],
        )
        .unwrap();
    });
    Ok(())
}

#[server]
pub async fn read_items() -> Result<Vec<Item>, ServerFnError> {
    backend::DB
        .with(|f| {
            let mut read = f.prepare("SELECT * FROM dooT").unwrap();
            let rows = read
                .query_map((), |row| {
                    Ok(Item {
                        id: row.get(0).unwrap(),
                        name: row.get(1).unwrap(),
                        checked: row.get(2).unwrap(),
                    })
                })
                .unwrap();
            rows.collect::<Result<Vec<_>, _>>()
        })
        .map_err(ServerFnError::new)
}
