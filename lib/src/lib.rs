use futures::FutureExt;
use leptos::*;
pub use leptos_tea_macros::*;
use smallvec::SmallVec;
use std::{
  future::Future,
  pin::Pin,
};

type CmdFut<Msg> = Pin<Box<dyn Future<Output = SmallVec<[Msg; 4]>>>>;

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

  pub fn cmd<Fut, I>(&mut self, cmd: Fut) -> &mut Self
  where
    Fut: Future<Output = I> + 'static,
    I: IntoIterator<Item = Msg>,
  {
    self
      .cmds
      .push(Box::pin(cmd.map(|i| i.into_iter().collect())));

    self
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
