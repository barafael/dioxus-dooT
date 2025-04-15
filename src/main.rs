use dioxus::prelude::*;
use rusqlite::Connection;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    id: u32,
    name: String,
    checked: bool,
}

fn main() {
    let con = Connection::open("./dooT.db3").unwrap();
    con.execute(
        "CREATE TABLE IF NOT EXISTS dooT ( id INTEGER PRIMARY KEY, name TEXT NOT NULL, checked BOOLEAN)",
        (),
    )
    .unwrap();

    dioxus::launch(dooT);
}

#[component]
pub fn dooT() -> Element {
    let item_name = use_signal(|| String::new());
    let mut items = use_signal(|| Vec::<Item>::new());

    let mut con = use_signal(|| Connection::open("./dooT.db3").unwrap());

    let add_item = move |item: String| {
        con.write()
            .execute(
                "INSERT INTO dooT (name, checked) VALUES(?1, ?2)",
                (&item, false),
            )
            .unwrap();
    };

    let delete_item = move |item: Item| {
        con.write()
            .execute("DELETE FROM dooT WHERE id=?1", [item.id])
            // .unwrap() // try to use this instead and see what errors you get.
            .unwrap();
    };

    let toggle_checked = move |id: u32| {
        con.write()
            .execute(
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
    };

    use_effect(move || {
        items.write().clear();
        let read = con.read();
        let mut read_statement = read.prepare("SELECT * FROM dooT").unwrap();
        let rows = read_statement
            .query_map((), |row| {
                Ok(Item {
                    id: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                    checked: row.get(2).unwrap(),
                })
            })
            .unwrap();

        for row in rows {
            let item = row.unwrap();
            items.write().push(item);
        }
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        div {
            h3 { "dooT" }
            ItemInput { item_name, add_item }
            ul {
                id: "theList",
                for item in items.iter() {
                    ItemElement { item: item.clone(), delete_item, toggle_checked }
                }
            }
        }
    }
}

#[component]
fn ItemInput(item_name: Signal<String>, add_item: Callback<String>) -> Element {
    rsx! {
        div {
            class: "header",
            input {
                type:  "text",
                id: "myInput",
                placeholder: "Name...",
                class: "input",
                value: item_name,
                oninput: move |event| {
                    item_name.set(event.value());
                },
                onkeydown: move |event| {
                    if event.code().to_string() == "Enter".to_string() {
                        add_item(item_name());
                        item_name.set(String::new());
                    }
                }
            }
        },
    }
}

#[component]
fn ItemElement(item: Item, delete_item: Callback<Item>, toggle_checked: Callback<u32>) -> Element {
    rsx! {
        li {
            onclick: move |_| { toggle_checked(item.id) },
            class: if item.checked {"checked"},
            label { { item.name.clone() } },
            span {
                class: "close",
                onclick: move |_event| {
                    delete_item(item.clone());
                },
                class: "delete-button",
                "Delete"
            }
        }
    }
}
