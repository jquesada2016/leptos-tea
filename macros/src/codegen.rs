use crate::model::{Field, Model};
use core::fmt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_quote;

trait FieldSliceExt: AsRef<[Field]> {
  fn to_update_model_fields(&self, is_named: bool) -> TokenStream {
    let this = self.as_ref();

    let fields = this.iter().map(
      |Field {
         vis,
         name,
         ty,
         is_nested_model,
       }| {
        let ty = if *is_nested_model {
          format_ty("Update", ty)
        } else {
          syn::parse_quote! { ::leptos_tea::leptos_reactive::RwSignal<#ty> }
        };

        if is_named {
          quote! { #vis #name: #ty }
        } else {
          quote! { #vis #ty }
        }
      },
    );

    if is_named {
      quote! { { #( #fields ),* } }
    } else {
      quote! { ( #( #fields ),* ); }
    }
  }

  fn to_view_model_fields(&self, is_named: bool) -> TokenStream {
    let this = self.as_ref();

    let fields = this.iter().map(
      |Field {
         vis,
         name,
         ty,
         is_nested_model,
       }| {
        let ty = if *is_nested_model {
          format_ty("View", ty)
        } else {
          syn::parse_quote! { ::leptos_tea::leptos_reactive::ReadSignal<#ty> }
        };

        if is_named {
          quote! { #vis #name: #ty }
        } else {
          quote! { #vis #ty }
        }
      },
    );

    if is_named {
      quote! { { #( #fields ),* } }
    } else {
      quote! { ( #( #fields ),* ); }
    }
  }
}

impl<T> FieldSliceExt for T where T: AsRef<[Field]> {}

pub fn codegen(
  Model {
    vis,
    name,
    generics,
    is_named,
    fields,
  }: Model,
) -> TokenStream {
  codegen_struct(vis, name, generics, is_named, fields)
}

fn codegen_struct(
  vis: syn::Visibility,
  name: syn::Ident,
  generics: syn::Generics,
  is_named: bool,
  fields: Vec<Field>,
) -> TokenStream {
  let update_struct = generate_model_struct(
    ModelStructKind::Update,
    &vis,
    &name,
    &generics,
    is_named,
    &fields,
  );

  let view_struct = generate_model_struct(
    ModelStructKind::View,
    &vis,
    &name,
    &generics,
    is_named,
    &fields,
  );

  let model_impl =
    generate_model_impl(&vis, &name, &generics, is_named, &fields);

  quote! {
    #update_struct

    #view_struct

    #model_impl
  }
}

fn format_ty(name: &str, ty: &syn::Type) -> syn::Type {
  let mut ty = ty.clone();

  if let syn::Type::Path(syn::TypePath { path, .. }) = &mut ty {
    let last_segment = path.segments.iter_mut().last().unwrap();

    last_segment.ident = format_ident!("{name}{}", last_segment.ident);
  } else {
    abort!(ty, "only path types are allowed")
  }

  ty
}

enum ModelStructKind {
  Update,
  View,
}

impl fmt::Display for ModelStructKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Update => f.write_str("Update"),
      Self::View => f.write_str("View"),
    }
  }
}

fn generate_model_struct(
  kind: ModelStructKind,
  vis: &syn::Visibility,
  name: &syn::Ident,
  generics: &syn::Generics,
  is_named: bool,
  fields: &[Field],
) -> TokenStream {
  let model_name = format_ident!("{kind}{name}");

  let model_fields = fields.iter().map(
    |Field {
       vis,
       name,
       ty,
       is_nested_model,
     }| {
      let ty = match kind {
        ModelStructKind::Update => {
          if *is_nested_model {
            format_ty(&kind.to_string(), ty)
          } else {
            parse_quote! { ::leptos_tea::leptos_reactive::RwSignal<#ty> }
          }
        }
        ModelStructKind::View => {
          if *is_nested_model {
            format_ty(&kind.to_string(), ty)
          } else {
            parse_quote! { ::leptos_tea::leptos_reactive::ReadSignal<#ty> }
          }
        }
      };

      if is_named {
        quote! { #vis #name: #ty }
      } else {
        quote! { #vis #ty }
      }
    },
  );

  let (_, type_generics, where_clause) = generics.split_for_impl();

  let model_fields = if is_named {
    quote! { #where_clause { #( #model_fields ),* } }
  } else {
    quote! { ( #( #model_fields ),* ) #where_clause ; }
  };

  quote! {
    #[derive(Clone, Copy)]
    #vis struct #model_name #type_generics #model_fields
  }
}

fn generate_model_impl(
  vis: &syn::Visibility,
  name: &syn::Ident,
  generics: &syn::Generics,
  is_named: bool,
  fields: &[Field],
) -> TokenStream {
  let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

  let split_fn_impl =
    generate_split_fn_impl(vis, name, generics, is_named, fields);
  let init_fn_impl = generate_init_fn_impl(vis, name, generics);

  quote! {
    impl #impl_generics #name #type_generics #where_clause {
      #split_fn_impl

      #init_fn_impl
    }
  }
}

fn generate_split_fn_impl(
  vis: &syn::Visibility,
  name: &syn::Ident,
  generics: &syn::Generics,
  is_named: bool,
  fields: &[Field],
) -> TokenStream {
  let update_model_name = format_ident!("Update{name}");
  let view_model_name = format_ident!("View{name}");

  let field_names = fields
    .iter()
    .enumerate()
    .map(|(i, field)| {
      if let Some(name) = &field.name {
        name.clone()
      } else {
        format_ident!("field_{i}")
      }
    })
    .collect::<Vec<_>>();

  let get_fields = if is_named {
    quote! { { #( #field_names ),* } }
  } else {
    quote! { ( #( #field_names ),* ) }
  };

  let split_model_fields = fields
    .iter()
    .zip(field_names.iter())
    .map(
      |(
        Field {
          is_nested_model, ..
        },
        field_name,
      )| {
        let read_name = format_ident!("__read_{field_name}");
        let write_name = format_ident!("__write_{field_name}");

        let split = if *is_nested_model {
          quote! { let (#read_name, #write_name) = #field_name.split(); }
        } else {
          quote! {
            let #write_name = ::leptos_tea::leptos_reactive::create_rw_signal(#field_name);
            let #read_name = #write_name.read_only();
          }
        };

        (split, read_name, write_name)
      },
    )
    .collect::<Vec<_>>();

  let split_model_fields_exprs =
    split_model_fields.iter().map(|(split, _, _)| split);

  let init_update_model_fields =
    split_model_fields.iter().map(|(_, _, write)| write);

  let init_view_model_fields =
    split_model_fields.iter().map(|(_, read, _)| read);

  let init_update_model_fields = if is_named {
    quote! { { #( #field_names: #init_update_model_fields ),* } }
  } else {
    quote! { ( #( #init_update_model_fields ),* ) }
  };

  let init_view_model_fields = if is_named {
    quote! { { #( #field_names: #init_view_model_fields ),* } }
  } else {
    quote! { ( #( #init_view_model_fields ),* ) }
  };

  let (_, type_generics, _) = generics.split_for_impl();

  quote! {
    #vis fn split(
      self,
    ) -> (#view_model_name #type_generics, #update_model_name #type_generics) {
      let Self #get_fields = self;

      #( #split_model_fields_exprs )*

      let __view_model = #view_model_name #init_view_model_fields;
      let __update_model = #update_model_name #init_update_model_fields;

      (__view_model, __update_model)
    }
  }
}

fn generate_init_fn_impl(
  vis: &syn::Visibility,
  name: &syn::Ident,
  generics: &syn::Generics,
) -> TokenStream {
  let update_model_name = format_ident!("Update{name}");
  let view_model_name = format_ident!("View{name}");

  let (_, type_generics, _) = generics.split_for_impl();

  quote! {
    #[track_caller]
    #vis fn init<Msg: ::core::default::Default + 'static>(
      self,
      update_fn: impl ::core::ops::Fn(
        #update_model_name #type_generics,
        Msg,
        ::leptos_tea::Cmd<Msg>,
      ) + 'static
    ) -> (#view_model_name #type_generics, ::leptos_tea::MsgDispatcher<Msg>) {
      let __update_fn = update_fn;

      let (__tx, mut __rx)
        = ::leptos_tea::futures::channel::mpsc::unbounded();

      __tx.unbounded_send(Msg::default()).unwrap();

      let __tx_store = ::leptos_tea::leptos_reactive::store_value(__tx);

      let (__view_model, __update_model) = self.split();

      ::leptos_tea::leptos_reactive::try_spawn_local_with_current_owner(
        async move {
          while let Some(msg)
            = ::leptos_tea::futures::StreamExt::next(&mut __rx).await
          {
            let __cmd_dispatcher = ::leptos_tea::Cmd::new(
              ::leptos_tea::leptos_reactive::Owner::current()
                .expect(
                  "`Model::init()` must be called  within \
                  the context of an `Owner`\n\
                  \n\
                  try calling `Model::init()` at the top level of \
                  the component definition"
                ),
              __tx_store,
            );

            __update_fn(__update_model, msg, __cmd_dispatcher);
          }
        },
        || {},
      );

      (__view_model, ::leptos_tea::MsgDispatcher::new(__tx_store))
    }
  }
}
