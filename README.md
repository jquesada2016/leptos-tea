The Elm Architecture for `leptos`.

This crate is a particular strategy for state management
in leptos`. It follows the Elm architecture, but not
strictly so, which allows mixing and matching with other state
management approaches.

First, let's look at an example.

# Example

**Note**: This example uses the `nightly` feature flag for
both `leptos_tea` and `leptos`.

```rust
use leptos::*;
use leptos_tea::Cmd;

#[derive(Default, leptos_tea::Model)]
struct CounterModel {
  counter: usize,
}

#[derive(Default)]
enum Msg {
  #[default]
  Init,
  Increment,
  Decrement,
}

fn update(model: UpdateCounterModel, msg: Msg, _: Cmd<Msg>) {
  match msg {
    Msg::Increment => model.counter.update(|c| *c += 1),
    Msg::Decrement => model.counter.update(|c| *c -= 1),
    Msg::Init => {}
  }
}

#[component]
fn Counter(cx: Scope) -> impl IntoView {
  let (model, msg_dispatcher) = CounterModel::default().init(cx, update);

  view! { cx,
    <h1>{model.counter}</h1>
   <button on:click=move |_| msg_dispatcher.dispatch(Msg::Decrement)>"-"</button>
   <button on:click=move |_| msg_dispatcher.dispatch(Msg::Increment)>"+"</button>
  }
}
```

In the above example, we're annotating `CounterModel` with
`leptos_tea::Model`, which will derive a few important things:

```rust
# use leptos::*;
# use leptos_tea::Cmd;

// Original struct, stays as-is
struct CounterModel {
  counter: usize,
}

// Model passed to the update function
struct UpdateCounterModel {
  counter: RwSignal<bool>,
}

// model passed to the component when you call `.init()`
struct ViewCounterModel {
  counter: ReadSignal<bool>,
}

impl CounterModel {
  // Initializes everything and starts listening for messages.
  // Msg::default() will be send to the update function when
  // called
  fn init<Msg: Default + 'static>(
    self,
    cx: Scope,
    update_fn: impl Fn(UpdateCounterModel, Msg, Cmd<Msg>),
  ) -> (ViewCounterModel, SignalSetter<Msg>) {
    /* ... */
# todo!()
  }
}
```

You first need to create your `CounterModel`, however you'd like.
In this case, we're using `Default`. Then you call `.init()`,
which will return a tuple containing the read-only model, as well
as a `MsgDispatcher`, which allows you to do `msg_dispatcher(Msg::Blah)`
on nightly, or `msg_dispatcher.dispatch(Msg::Blah)` on stable.

And that's how this crate and state management approach works.

# Model nesting

Models can be nested inside one another like thus:

```rust
#[derive(leptos_tea::Model)]
struct Model {
  #[model]
  inner_model: InnerModel,
}

#[derive(leptos_tea::Model)]
struct InnerModel(/* ... */);
```

# Limitations

`leptos_tea::Model` currently only supports tuple and field structs.
Enum support will be added soon.

# Features

- `nightly`: Implements `Fn(Msg)` for `MsgDispatcher`.