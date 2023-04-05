#![cfg_attr(not(feature = "stable"), feature(unboxed_closures, fn_traits))]
#![deny(missing_docs)]

//! The Elm Architecture for [`leptos`].
//!
//! This crate is a particular strategy for state management
//! in [`leptos`]. It follows the Elm architecture, but not
//! strictly so, which allows mixing and matching with other state
//! management approaches.
//!
//! First, let's look at an example.
//!
//! # Example
//! ```rust
//! use leptos::*;
//! use leptos_tea::Cmd;
//!
//! #[derive(Default, leptos_tea::Model)]
//! struct CounterModel {
//!   counter: usize,
//! }
//!
//! #[derive(Default)]
//! enum Msg {
//!   Increment,
//!   Decrement,
//!   #[default]
//!   Init,
//! }
//!
//! fn update(model: UpdateCounterModel, msg: &Msg, _: Cmd<Msg>) {
//!   match msg {
//!     Msg::Increment => model.counter.update(|c| *c += 1),
//!     Msg::Decrement => model.counter.update(|c| *c -= 1),
//!     Msg::Init => {}
//!   }
//! }
//!
//! #[component]
//! fn Counter(cx: Scope) -> impl IntoView {
//!   let (model, msg_dispatcher) = CounterModel::default().init(cx, update);
//!
//!   view! { cx,
//!     <h1>{model.counter}</h1>
//!    <button on:click=move |_| msg_dispatcher(Msg::Decrement)>"-"</button>
//!    <button on:click=move |_| msg_dispatcher(Msg::Increment)>"+"</button>
//!   }
//! }
//! ```
//!
//! In the above example, we're annotating `CounterModel` with
//! `leptos_tea::Model`, which will derive a few important things:
//!
//! ```rust
//! # use leptos::*;
//! # use leptos_tea::Cmd;
//!
//! // Original struct, stays as-is
//! struct CounterModel {
//!   counter: usize,
//! }
//!
//! // Model passed to the update function
//! struct UpdateCounterModel {
//!   counter: RwSignal<bool>,
//! }
//!
//! // model passed to the component when you call `.init()`
//! struct ViewCounterModel {
//!   counter: ReadSignal<bool>,
//! }
//!
//! impl CounterModel {
//!   // Initializes everything and starts listening for messages.
//!   // Msg::default() will be send to the update function when
//!   // called
//!   fn init<Msg: Default + 'static>(
//!     self,
//!     cx: Scope,
//!     update_fn: impl Fn(UpdateCounterModel, &Msg, Cmd<Msg>),
//!   ) -> (ViewCounterModel, SignalSetter<Msg>) {
//!     /* ... */
//! # todo!()
//!   }
//! }
//! ```
//!
//! You first need to create your `CounterModel`, however you'd like.
//! In this case, we're using `Default`. Then you call `.init()`,
//! which will return a tuple containing the read-only model, as well
//! as a `SignalSetter`, which allows you to do `msg_dispatcher(Msg::Blah)`
//! on nightly, or `msg_dispatcher.set(Msg::Blah)` on stable.
//!
//! And that's how this crate and state management approach works.
//!
//! # Model nesting
//!
//! Models can be nested inside one another like thus:
//!
//! ```rust
//! #[derive(leptos_tea::Model)]
//! struct Model {
//!   #[model]
//!   inner_model: InnerModel,
//! }
//!
//! #[derive(leptos_tea::Model)]
//! struct InnerModel(/* ... */);
//! ```
//!
//! **Important Node**: Although this _can_ be done, it is not
//! recommended, because it leads to nested `.update()`/`.with`
//! calls for each level of nesting. Instead, try and break out each
//! nested model into it's own independent model, view, update. Nevertheless,
//! sometimes this isn't desired or worth it, so the option is there in case
//! you need it.

//!
//! # Limitations
//!
//! `leptos_tea::Model` currently only supports tuple and field structs.
//! Support will be added soon.

use futures::FutureExt;
use leptos_reactive::*;
pub use leptos_tea_macros::*;
use smallvec::SmallVec;
use std::{
  future::Future,
  pin::Pin,
};

type CmdFut<Msg> = Pin<Box<dyn Future<Output = SmallVec<[Msg; 4]>>>>;

/// Command manager that allows dispatching messages and running
/// asynchronous operations.
pub struct Cmd<Msg: 'static> {
  msg_dispatcher: SignalSetter<Msg>,
  msgs: SmallVec<[Msg; 4]>,
  cmds: SmallVec<[CmdFut<Msg>; 4]>,
}

impl<Msg: 'static> Cmd<Msg> {
  /// Creates a new [`Cmd`] queue.
  ///
  /// You shouldn't need to use this, as it will be
  /// code generated by the [`Model`] derive macro.
  pub fn new(msg_dispatcher: SignalSetter<Msg>) -> Self {
    Self {
      msg_dispatcher,
      cmds: Default::default(),
      msgs: Default::default(),
    }
  }

  /// Adds this message to the command queue which will be dispatched
  /// to the update function on the next microtask.
  pub fn msg(&mut self, msg: Msg) {
    self.msgs.push(msg);
  }

  /// Same as [`Cmd::msg`], but allows adding multiple messages at once.
  pub fn batch_msgs<I: IntoIterator<Item = Msg>>(&mut self, msgs: I) {
    self.msgs.extend(msgs);
  }

  /// Adds a command to the queue that will be executed when
  /// this struct is dropped.
  pub fn cmd<Fut, I>(&mut self, cmd: Fut)
  where
    Fut: Future<Output = I> + 'static,
    I: IntoIterator<Item = Msg>,
  {
    self
      .cmds
      .push(Box::pin(cmd.map(|i| i.into_iter().collect())));
  }

  /// Manually perform all commands and dispatch messages now rather
  /// than when dropping.
  pub fn perform(&mut self) {
    // Will perform actions on drop, so pseudo-clone it
    // and just let it drop
    Self {
      msg_dispatcher: self.msg_dispatcher,
      msgs: core::mem::take(&mut self.msgs),
      cmds: core::mem::take(&mut self.cmds),
    };
  }
}

/// Creates a new [`Cmd`] struct to send dispatch messages
/// to the `update` function.
impl<Msg: 'static> Clone for Cmd<Msg> {
  fn clone(&self) -> Self {
    Self {
      msg_dispatcher: self.msg_dispatcher,
      msgs: Default::default(),
      cmds: Default::default(),
    }
  }
}

impl<Msg: 'static> Drop for Cmd<Msg> {
  fn drop(&mut self) {
    let msg_dispatcher = self.msg_dispatcher;

    for cmds in std::mem::take(&mut self.cmds) {
      spawn_local(async move {
        let mut cmds = cmds.await.into_iter();

        if let Some(msg) = cmds.next() {
          msg_dispatcher(msg);
        }

        for msg in cmds {
          spawn_local(async move { msg_dispatcher(msg) });
        }
      });
    }

    for msg in std::mem::take(&mut self.msgs) {
      queue_microtask(move || msg_dispatcher.set(msg));
    }
  }
}

/// Used to send messages to the `update` function.
pub struct MsgDispatcher<Msg: 'static>(WriteSignal<Msg>);

impl<Msg: 'static> From<WriteSignal<Msg>> for MsgDispatcher<Msg> {
  fn from(writer: WriteSignal<Msg>) -> Self {
    Self(writer)
  }
}

impl<Msg: 'static> Clone for MsgDispatcher<Msg> {
  fn clone(&self) -> Self {
    Self(self.0)
  }
}

impl<Msg: 'static> Copy for MsgDispatcher<Msg> {}

impl<Msg: 'static> SignalSet<Msg> for MsgDispatcher<Msg> {
  fn set(&self, new_value: Msg) {
    let dispatcher = self.0;

    queue_microtask(move || dispatcher.set(new_value));
  }

  fn try_set(&self, new_value: Msg) -> Option<Msg> {
    self.0.try_set(new_value)
  }
}

#[cfg(not(feature = "stable"))]
impl<Msg> FnOnce<(Msg,)> for MsgDispatcher<Msg> {
  type Output = ();

  extern "rust-call" fn call_once(self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

#[cfg(not(feature = "stable"))]
impl<Msg> FnMut<(Msg,)> for MsgDispatcher<Msg> {
  extern "rust-call" fn call_mut(&mut self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

#[cfg(not(feature = "stable"))]
impl<Msg> Fn<(Msg,)> for MsgDispatcher<Msg> {
  extern "rust-call" fn call(&self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

impl<Msg> MsgDispatcher<Msg> {
  /// Sends a message to the update function.
  ///
  /// This is the same as calling `msg_dispatcher.set(msg)`, or on
  /// nightly, `msg_dispatcher(msg)`.
  #[inline]
  pub fn dispatch(self, msg: Msg) {
    self.set(msg);
  }

  /// Dispatches the message immediately, rather than waiting for
  /// the next micro-task.
  pub fn dispatch_immediate(self, msg: Msg) {
    self.0.set(msg);
  }

  /// Batches multiple messages together.
  ///
  /// All messages are sent one after another.
  pub fn batch<I>(self, msgs: I)
  where
    I: IntoIterator<Item = Msg>,
  {
    for msg in msgs {
      self.dispatch(msg);
    }
  }
}
