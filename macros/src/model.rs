pub struct Model {
  pub vis: syn::Visibility,
  pub name: syn::Ident,
  pub generics: syn::Generics,
  pub is_named: bool,
  pub fields: Vec<Field>,
}

impl From<syn::DeriveInput> for Model {
  fn from(
    syn::DeriveInput {
      ident: name,
      vis,
      generics,
      data,
      ..
    }: syn::DeriveInput,
  ) -> Self {
    match data {
      syn::Data::Struct(syn::DataStruct { fields, .. }) => {
        if matches!(fields, syn::Fields::Unit) {
          abort!(name, "unit structs are not supported");
        }

        Model {
          vis,
          name,
          generics,
          is_named: matches!(fields, syn::Fields::Named(_)),
          fields: fields.into_iter().map(Field::from).collect(),
        }
      }
      syn::Data::Enum(e) => abort!(e.enum_token, "enums are not supported"),
      syn::Data::Union(union) => {
        abort!(union.union_token, "unions are not supported")
      }
    }
  }
}

pub struct Field {
  pub vis: syn::Visibility,
  pub name: Option<syn::Ident>,
  pub ty: syn::Type,
  pub is_nested_model: bool,
}

impl From<syn::Field> for Field {
  fn from(
    syn::Field {
      attrs,
      vis,
      ident: name,
      ty,
      ..
    }: syn::Field,
  ) -> Self {
    Self {
      vis,
      name,
      ty,
      is_nested_model: is_nested_model(&attrs),
    }
  }
}

/// Derives all the goodness neaded for making some leptos tea.
pub fn model(ast: syn::DeriveInput) -> Model {
  Model::from(ast)
}

fn is_nested_model(attrs: &[syn::Attribute]) -> bool {
  attrs
    .iter()
    .any(|attr| *attr == syn::parse_quote!(#[model]))
}
