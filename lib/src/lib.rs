use leptos::*;
pub use leptos_tea_macros::*;
use smallvec::SmallVec;
use std::{
  future::Future,
  pin::Pin,
};

pub type CmdFut<Msg> = Pin<Box<dyn Future<Output = Option<Msg>>>>;

pub struct Cmd<Msg: 'static> {
  msg_dispatcher: SignalSetter<Msg>,
  msgs: SmallVec<[Msg; 4]>,
  cmds: SmallVec<[CmdFut<Msg>; 4]>,
}

impl<Msg: 'static> Cmd<Msg> {
  pub fn new(msg_dispatcher: SignalSetter<Msg>) -> Self {
    Self {
      msg_dispatcher,
      cmds: Default::default(),
      msgs: Default::default(),
    }
  }

  pub fn msg(&mut self, msg: Msg) -> &mut Self {
    self.msgs.push(msg);

    self
  }

  pub fn batch_msgs<I: IntoIterator<Item = Msg>>(
    &mut self,
    msgs: I,
  ) -> &mut Self {
    self.msgs.extend(msgs);

    self
  }

  pub fn cmd<Fut: Future<Output = Option<Msg>> + 'static>(
    &mut self,
    cmd: Fut,
  ) -> &mut Self {
    self.cmds.push(Box::pin(cmd));

    self
  }
}

impl<Msg: 'static> Drop for Cmd<Msg> {
  fn drop(&mut self) {
    let msg_dispatcher = self.msg_dispatcher;

    for msg in std::mem::take(&mut self.msgs) {
      queue_microtask(move || msg_dispatcher.set(msg));
    }

    for cmd in std::mem::take(&mut self.cmds) {
      spawn_local(async move {
        if let Some(msg) = cmd.await {
          msg_dispatcher.set(msg);
        }
      });
    }
  }
}
