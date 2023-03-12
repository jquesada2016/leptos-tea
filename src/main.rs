#[macro_use]
extern crate leptos_tea;

use leptos::*;

fn main() {
  mount_to_body(|cx| view! { cx, <App /> })
}

declare_model! {
  #[derive(Default)]
  struct Model {
    count: isize,
  }
}

#[derive(Clone, Copy, Debug, Default)]
enum Msg {
  Increment,
  Decrement,
  #[default]
  Noop,
}

fn update(model: UpdateModel, msg: &Msg) {
  match msg {
    Msg::Increment => model.count.update(|c| *c += 1),
    Msg::Decrement => model.count.update(|c| *c -= 1),
    Msg::Noop => {}
  }
}

#[component]
fn App(cx: Scope) -> impl IntoView {
  let (model, msg_dispatcher) = Model::default().init(cx, update);

  view! { cx,
    <h1>{model.count}</h1>
    <button on:click=move |_| msg_dispatcher(Msg::Decrement)>"-"</button>
    <button on:click=move |_| msg_dispatcher(Msg::Increment)>"+"</button>
  }
}
