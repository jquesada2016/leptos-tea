#![cfg_attr(feature = "nightly", feature(unboxed_closures, fn_traits))]
#![deny(missing_docs)]

//! The Elm Architecture for [`leptos_reactive`] apps.
//!
//! This crate is a particular strategy for state management
//! in apps that use [`leptos_reactive`]. It follows the Elm architecture, but not
//! strictly so, which allows mixing and matching with other state
//! management approaches.
//!
//! First, let's look at an example.
//!
//! # Example
//!
//! **Note**: This example uses the `nightly` feature flag for
//! both `leptos_tea` and `leptos`.
//!
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
//!   #[default]
//!   Init,
//!   Increment,
//!   Decrement,
//! }
//!
//! fn update(model: UpdateCounterModel, msg: Msg, _: Cmd<Msg>) {
//!   match msg {
//!     Msg::Increment => model.counter.update(|c| *c += 1),
//!     Msg::Decrement => model.counter.update(|c| *c -= 1),
//!     Msg::Init => {}
//!   }
//! }
//!
//! #[component]
//! fn Counter() -> impl IntoView {
//!   let (model, msg_dispatcher) = CounterModel::default().init(update);
//!
//!   view! {
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
//!     update_fn: impl Fn(UpdateCounterModel, Msg, Cmd<Msg>),
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
//! as a `MsgDispatcher`, which allows you to do `msg_dispatcher(Msg::Blah)`
//! on nightly, or `msg_dispatcher.dispatch(Msg::Blah)` on stable.
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
//! # Limitations
//!
//! `leptos_tea::Model` currently only supports tuple and field structs.
//! Enum support will be added soon.
//!
//! # Features
//!
//! - `nightly`: Implements `Fn(Msg)` for [`MsgDispatcher`].

#[doc(hidden)]
pub use futures;
use futures::{
  channel::mpsc::UnboundedSender,
  FutureExt,
  SinkExt,
};
#[doc(hidden)]
pub use leptos_reactive;
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
  msg_dispatcher: StoredValue<UnboundedSender<Msg>>,
  msgs: SmallVec<[Msg; 4]>,
  cmds: SmallVec<[CmdFut<Msg>; 4]>,
}

impl<Msg: 'static> Cmd<Msg> {
  #[doc(hidden)]
  ///
  /// You shouldn't need to use this, as it will be
  /// code generated by the [`Model`] derive macro.
  pub fn new(msg_dispatcher: StoredValue<UnboundedSender<Msg>>) -> Self {
    Self {
      msg_dispatcher,
      cmds: Default::default(),
      msgs: Default::default(),
    }
  }

  /// Adds this message to the command queue which will be dispatched
  /// to the update function on [`Drop`] or on [`Cmd::perform`].
  pub fn msg(&mut self, msg: Msg) {
    self.msgs.push(msg);
  }

  /// Same as [`Cmd::msg`], but allows adding multiple messages at once.
  pub fn batch_msgs<I: IntoIterator<Item = Msg>>(&mut self, msgs: I) {
    self.msgs.extend(msgs);
  }

  /// Adds an asynchronous task to the queue that will be executed when
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

/// Executes all commands when dropped. Use [`Cmd::perform`]
/// to force this to happen before `Cmd` drops.
impl<Msg: 'static> Drop for Cmd<Msg> {
  fn drop(&mut self) {
    let msg_dispatcher = self.msg_dispatcher.get_value();

    for cmds in std::mem::take(&mut self.cmds) {
      let mut msg_dispatcher = msg_dispatcher.clone();

      spawn_local(async move {
        let mut cmds = cmds.await.into_iter();

        if let Some(msg) = cmds.next() {
          msg_dispatcher.send(msg).await.unwrap();
        }

        for msg in cmds {
          let mut msg_dispatcher = msg_dispatcher.clone();

          spawn_local(async move { msg_dispatcher.send(msg).await.unwrap() });
        }
      });
    }

    for msg in std::mem::take(&mut self.msgs) {
      let mut msg_dispatcher = msg_dispatcher.clone();

      spawn_local(async move {
        msg_dispatcher.send(msg).await.unwrap();
      });
    }
  }
}

/// Used to send messages to the `update` function.
pub struct MsgDispatcher<Msg: 'static>(StoredValue<UnboundedSender<Msg>>);

impl<Msg: 'static> Clone for MsgDispatcher<Msg> {
  fn clone(&self) -> Self {
    Self(self.0)
  }
}

impl<Msg: 'static> Copy for MsgDispatcher<Msg> {}

#[cfg(feature = "nightly")]
impl<Msg> FnOnce<(Msg,)> for MsgDispatcher<Msg> {
  type Output = ();

  extern "rust-call" fn call_once(self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

#[cfg(feature = "nightly")]
impl<Msg> FnMut<(Msg,)> for MsgDispatcher<Msg> {
  extern "rust-call" fn call_mut(&mut self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

#[cfg(feature = "nightly")]
impl<Msg> Fn<(Msg,)> for MsgDispatcher<Msg> {
  extern "rust-call" fn call(&self, args: (Msg,)) -> Self::Output {
    self.dispatch(args.0);
  }
}

impl<Msg> MsgDispatcher<Msg> {
  #[doc(hidden)]
  pub fn new(msg_dispatcher: StoredValue<UnboundedSender<Msg>>) -> Self {
    Self(msg_dispatcher)
  }

  /// Dispatches the message to the update function.
  ///
  /// Does not immediately send the value, rather it waits for
  /// the next micro-task. This is done to avoid panics within
  /// the leptos runtime. If you need to send the message
  /// immediately, refer to [`MsgDispatcher::dispatch_immediate`].
  ///
  /// This is the same as calling  `msg_dispatcher(msg)`
  /// on nightly.
  pub fn dispatch(self, msg: Msg) {
    let mut msg_dispatcher = self.0.get_value();

    spawn_local(async move {
      msg_dispatcher.send(msg).await.unwrap();
    });
  }

  /// Dispatches the message immediately, rather than waiting for
  /// the next micro-task.
  pub fn dispatch_immediate(self, msg: Msg) {
    let msg_dispatcher = self.0.get_value();

    msg_dispatcher.unbounded_send(msg).unwrap();
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
