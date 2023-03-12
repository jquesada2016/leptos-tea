#[macro_export]
macro_rules! declare_model {
  (
    $( #[$model_meta:meta] )*
    $vis:vis struct $model:ident {
      $(
        $( #[$field_meta:meta] )*
        $field_vis:vis $field_name:ident : $field_ty:ty
      ),* $(,)?
    }
  ) => {
    paste::paste! {
      $( #[$model_meta] )*
      $vis struct $model {
        $(
          $( #[$field_meta] )*
          $field_vis $field_name : $field_ty
        ),*
      }

      $vis struct [<Update $model>] {
        $(
          $field_vis $field_name: $crate::__leptos::WriteSignal<$field_ty>
        ),*
      }

      impl ::core::clone::Clone for [<Update $model>] {
        fn clone(&self) -> Self {
          Self {
            $(
              $field_name: $crate::__core::clone::Clone::clone(&self.$field_name)
            ),*
          }
        }
      }

      impl Copy for [<Update $model>] {}

      $vis struct [<View $model>] {
        $(
          $field_vis $field_name: $crate::__leptos::ReadSignal<$field_ty>
        ),*
      }

      impl ::core::clone::Clone for [<View $model>] {
        fn clone(&self) -> Self {
          Self {
            $(
              $field_name: self.$field_name.clone()
            ),*
          }
        }
      }

      impl Copy for [<View $model>] {}

      impl $model {
        $vis fn init<Msg: Default + 'static>(
          self,
          cx: $crate::__leptos::Scope,
          update_fn: impl Fn([<Update $model>], &Msg) + 'static
        ) -> ([<View $model>], $crate::MsgDispatcher<Msg>) {
          let Self {
            $( $field_name ),*
          } = self;

          let (msg, msg_dispatcher)
            = $crate::__leptos::create_signal(cx, Msg::default());

          $(
            let ($field_name, [<set_ $field_name>])
              = $crate::__leptos::create_signal(cx, $field_name);
          )*

          let update_model = [<Update $model>] {
            $( $field_name: [<set_ $field_name>] ),*
          };

          let view_model = [<View $model>] {
            $( $field_name ),*
          };

          $crate::__leptos::create_effect(cx, move |_| {
            msg.with(|msg| update_fn(update_model, msg));
          });

          (
            view_model,
            msg_dispatcher.into(),
          )
        }
      }
    }
  };
}

pub type MsgDispatcher<Msg> = SignalSetter<Msg>;
#[doc(hidden)]
pub use core as __core;
#[doc(hidden)]
pub use leptos as __leptos;
use leptos::*;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
struct ModelUpdate<Msg> {
  counter: WriteSignal<isize>,
  _msg: PhantomData<Msg>,
}

#[derive(Clone, Copy)]
struct ModelView<Msg> {
  counter: ReadSignal<isize>,
  _msg: PhantomData<Msg>,
}

#[derive(Clone, Copy)]
struct Model<Msg>(PhantomData<Msg>);

impl<Msg: Copy + Default + 'static> Model<Msg> {
  #[allow(clippy::new_ret_no_self)]
  pub fn new(
    cx: Scope,
    update_fn: impl Fn(ModelUpdate<Msg>, &Msg) + 'static,
  ) -> (ModelView<Msg>, MsgDispatcher<Msg>) {
    let (counter, set_counter) = create_signal(cx, Default::default());
    let (msg, msg_dispatcher) = create_signal(cx, Msg::default());

    let update_model = ModelUpdate {
      counter: set_counter,
      _msg: PhantomData,
    };

    create_effect(cx, move |_| {
      msg.with(|msg| {
        update_fn(update_model, msg);
      });
    });

    (
      ModelView {
        counter,
        _msg: PhantomData,
      },
      msg_dispatcher.into(),
    )
  }
}

#[derive(Clone, Copy, Debug)]
enum Msg {
  Increment,
  Decrement,
}

#[component]
fn Counter(cx: Scope) -> impl IntoView {}

#[cfg(test)]
mod tests {
  use super::*;
  use ::typed_builder::TypedBuilder;

  #[test]
  fn compiles() {
    declare_model! {
      #[derive(TypedBuilder)]
      struct Model {
        counter: isize,
      }
    }
  }
}
