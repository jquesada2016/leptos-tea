use leptos::*;
use leptos_tea::Model;

fn main() {}

#[derive(Model, Default)]
struct Model<T>(T);

fn _comp(cx: Scope) -> impl IntoView {
  let (model, msg) = Model::<isize>::default().init(cx, |_, _: &()| {});
}
