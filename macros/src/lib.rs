// macro_rules! declare_model {
//   (
//     $( #[$model_meta:meta] )*
//     $vis:vis struct $model:ident {
//       $(
//         $( #[$field_meta:meta] )*
//         $field_vis:vis $field_name:ident : $field_ty:ty
//       ),* $(,)?
//     }
//   ) => {
//     paste::paste! {
//       $( #[$model_meta] )*
//       $vis struct $model {
//         $(
//           $( #[$field_meta] )*
//           $field_vis $field_name : $field_ty
//         ),*
//       }

//       #[derive(Clone, Copy)]
//       $vis struct [<Update $model>] {
//         $(
//           $field_vis $field_name: leptos_reactive::WriteSignal<$field_ty>
//         ),*
//       }

//       #[derive(Clone, Copy)]
//       $vis struct [<View $model>] {
//         $(
//           $field_vis $field_name: leptos_reactive::ReadSignal<$field_ty>
//         ),*
//       }

//       impl $model {
//         $vis fn init<Msg: Default + 'static>(
//           self,
//           cx: leptos_reactive::Scope,
//           update_fn: impl Fn([<Update $model>], &Msg) + 'static
//         ) -> ([<View $model>], $crate::MsgDispatcher<Msg>) {
//           let Self {
//             $( $field_name ),*
//           } = self;

//           let (msg, msg_dispatcher)
//             = leptos_reactive::create_signal(cx, Msg::default());

//           $(
//             let ($field_name, [<set_ $field_name>])
//               = leptos_reactive::create_signal(cx, $field_name);
//           )*

//           let update_model = [<Update $model>] {
//             $( $field_name: [<set_ $field_name>] ),*
//           };

//           let view_model = [<View $model>] {
//             $( $field_name ),*
//           };

//           leptos_reactive::create_effect(cx, move |_| {
//             msg.with(|msg| update_fn(update_model, msg));
//           });

//           (
//             view_model,
//             msg_dispatcher.into(),
//           )
//         }
//       }
//     }
//   };

//   (
//     $( #[$model_meta:meta] )*
//     $vis:vis struct $model:ident (
//       $(
//         $( #[$field_meta:meta] )*
//         $field_vis:vis $field_ty:ty
//       ),* $(,)?
//     )
//   ) => {
//     paste::paste! {
//       $( #[$model_meta] )*
//       $vis struct $model (
//         $(
//           $( #[$field_meta] )*
//           $field_vis $field_ty
//         ),*
//       )

//       $vis struct [<Update $model>] (
//         $(
//           $field_vis leptos_reactive::WriteSignal<$field_ty>
//         ),*
//       )

//       impl ::core::clone::Clone for [<Update $model>] {
//         fn clone(&self) -> Self {
//           let Self(
//             $( $field ),*
//           ) = self;

//           Self(
//             $( core::clone::Clone::clone($field) ),*
//           )
//         }
//       }

//       impl Copy for [<Update $model>] {}

//       $vis struct [<View $model>] {
//         $(
//           $field_vis $field_name: leptos_reactive::ReadSignal<$field_ty>
//         ),*
//       }

//       impl ::core::clone::Clone for [<View $model>] {
//         fn clone(&self) -> Self {
//           Self {
//             $(
//               $field_name: self.$field_name.clone()
//             ),*
//           }
//         }
//       }

//       impl Copy for [<View $model>] {}

//       impl $model {
//         $vis fn init<Msg: Default + 'static>(
//           self,
//           cx: leptos_reactive::Scope,
//           update_fn: impl Fn([<Update $model>], &Msg) + 'static
//         ) -> ([<View $model>], MsgDispatcher<Msg>) {
//           let Self {
//             $( $field_name ),*
//           } = self;

//           let (msg, msg_dispatcher)
//             = leptos_reactive::create_signal(cx, Msg::default());

//           $(
//             let ($field_name, [<set_ $field_name>])
//               = leptos_reactive::create_signal(cx, $field_name);
//           )*

//           let update_model = [<Update $model>] {
//             $( $field_name: [<set_ $field_name>] ),*
//           };

//           let view_model = [<View $model>] {
//             $( $field_name ),*
//           };

//           leptos_reactive::create_effect(cx, move |_| {
//             msg.with(|msg| update_fn(update_model, msg));
//           });

//           (
//             view_model,
//             msg_dispatcher.into(),
//           )
//         }
//       }
//     }
//   };
// }

// use core as __core;
// #[doc(hidden)]
// use leptos_reactive::SignalWith;
// #[doc(hidden)]
// use std::marker::PhantomData;

// type MsgDispatcher<Msg> = leptos_reactive::SignalSetter<Msg>;

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use ::typed_builder::TypedBuilder;

//   #[test]
//   fn compiles() {
//     declare_model! {
//       #[derive(TypedBuilder)]
//       struct Model {
//         counter: isize,
//       }
//     }
//   }
// }

#[macro_use]
extern crate proc_macro_error;

mod codegen;
mod model;

use proc_macro_error::proc_macro_error;

#[proc_macro_derive(Model, attributes(model))]
#[proc_macro_error]
pub fn model(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = syn::parse_macro_input!(stream as syn::DeriveInput);

  let model = model::model(ast);

  codegen::codegen(model).into()
}
