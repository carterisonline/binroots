use convert_case::{Case, Casing};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// # binroots_enum
/// A procedural macro attribute that enables serialization, default and debug operations for an enum.
/// Usage
/// `#[binroots_enum]` can be applied to enums. The attribute generates an implementation of [`binroots::Serialize`][brserialize] (re-exported from serde::Serialize), and attempts to automatically pick the default value.
/// ```rust
/// use binroots::binroots_enum;
///
/// #[binroots_enum]
/// pub enum MyEnum {
///     VariantA,
///     VariantB,
///     #[default]
///     VariantC,
/// }
/// ```
/// Notice how we're using `#[default]` to mark the default variant of `MyEnum`. It's also possible for `#[binroots_enum]` to automatically pick the default variant:
/// ```
/// use binroots::binroots_enum;
///
/// #[binroots_enum]
/// pub enum MyEnum {
///     None, // Automatically chosen to be the default value
///           // #[binroots_enum] automatically picks the first instance of either "None", "Nothing", "Default" or "Empty" to be default.
///     VariantA,
///     VariantB,
/// }
///
/// #[binroots_enum(manual)]
/// pub enum MyManualEnum {
///     None, // Not selected as the default variant because we use the `manual` annotation
///     #[default]
///     VariantA,
///     VariantB
/// }
/// ```
/// The generated code includes a new implementation of the input struct with the following changes:
///     - `derive`s [`Debug`], [`Default`], [`binroots::Serialize`][brserialize]
///     - Adds a `new` method to the struct, which constructs a new instance of the struct from its fields.
///     - A `#[default]` marker inserted wherever possible, overrided by the `manual` annotation
// Example
/// ```rust
/// use binroots::binroots_enum;
/// use binroots::save::{RootType, Save};
///
/// #[binroots_enum]
/// pub enum Activity {
///     Nothing,
///     Playing(String),
///     Watching(String),
/// }
///
/// fn main() {
///     let activity = Activity::Playing("bideo games".into());
///
///     activity.save("activity", RootType::InMemory).unwrap(); // Saves the enum to the disk
/// }
/// ```
/// [brserialize]: https://docs.rs/binroots/latest/binroots/trait.Serialize.html
#[proc_macro_attribute]
pub fn binroots_enum(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let vis = input.vis;
    let ident = input.ident;
    let attrs = input.attrs;
    let generics = input.generics;

    let mut found = None;
    let mut manual = false;

    for a in attr {
        manual = a.to_string() == "manual"
    }

    let variants = if let syn::Data::Enum(syn::DataEnum { variants, .. }) = input.data {
        if manual {
            variants.into_iter().map(|v| quote!(#v)).collect::<Vec<_>>()
        } else {
            variants
                .into_iter()
                .enumerate()
                .map(|(i, v)| {
                    if (v.ident == "None"
                        || v.ident == "Nothing"
                        || v.ident == "Empty"
                        || v.ident == "Default")
                        && found.is_none()
                    {
                        found = Some(i)
                    }

                    if let Some(c) = found {
                        if c == i {
                            quote! {
                                #[default]
                                #v
                            }
                        } else {
                            quote!(#v)
                        }
                    } else {
                        quote!(#v)
                    }
                })
                .collect::<Vec<_>>()
        }
    } else {
        panic!("#[binroots_enum] only supports enums.")
    };

    let output = quote! {
        #[derive(Debug, Default, binroots::Serialize)]
        #( #attrs )*
        #vis enum #ident #generics {
            #(

                #variants
            ),*
        }
    };

    output.into()
}

/// # binroots_struct
/// A procedural macro attribute that enables serialization and structured file-saving for it and its fields.
/// Usage
/// `#[binroots_struct]` can be applied to any named struct with named fields. The attribute generates an implementation of [`binroots::Serialize`][brserialize] (re-exported from serde::Serialize) for the struct, as well as a set of methods for serializing, deserializing, and saving instances of the struct to disk.
/// ```rust
/// use binroots::binroots_struct;
///
/// #[binroots_struct] // Saves to `/tmp/<CARGO_PKG_NAME>/my-struct/` on Unix
/// pub struct MyStruct {
///     field1: i32,
///     field2: String,
///     field3: bool,
/// }
///
/// // --- OR ---
///
/// #[binroots_struct(persistent)] // Saves to `$HOME/.cache/<CARGO_PKG_NAME>/my-persistent-struct/` on Unix
/// pub struct MyPersistentStruct {
///     field1: i32,
///     field2: String,
///     field3: bool,
/// }
/// ```
/// The generated code includes a new implementation of the input struct with the following changes:
///     - Wraps each field in a [`binroots::field::BinrootsField`][brfield]
///     - `derive`s [`Debug`] and [`binroots::Serialize`][brserialize]
///     - Generates `Self::ROOT_FOLDER`, the kebab-case name of the struct
///     - Adds a `new` method to the struct, which constructs a new instance of the struct from its fields.
///     - Adds a `save` method to the struct, which serializes the struct and saves it to disk using the [`binroots::save::Save`][brsave] trait, saving to `Self::ROOT_FOLDER`.
///     - A [`Default`] implementation is added to the struct, which constructs a default instance of the struct with default values for all fields.
///
// Example
/// ```rust
/// use binroots::binroots_struct;
/// use binroots::save::RootType;
///
/// #[binroots_struct] // Root save path on Unix is `/tmp/<CARGO_PKG_NAME>/person/` because we didn't annotate with `#[binroots_struct(persistent)]`
/// pub struct Person {
///     name: String,
///     gender: String,
///     age: u8,
///     email: Option<String>,
/// }
///
/// fn main() {
///     let mut person = Person::new(
///         "Alex".into(),
///         "Male".into(),
///         42,
///         Some("alex@example.com".into()),
///     );
///
///     // We need to dereference because `person.alice` is `binroots::field::BinrootsField<"name", u8>`
///     *person.name = "Alice".into();
///     *person.gender = "Female".into();
///
///     person.save().unwrap(); // Saves the entire struct to the disk
///
///     *person.email = Some("alice@example.com".into());
///     person.email.save(Person::ROOT_FOLDER, RootType::InMemory).unwrap(); // Saves only person.email to the disk in its appropriate location
/// }
/// ```
/// [brserialize]: https://docs.rs/binroots/latest/binroots/trait.Serialize.html
/// [brfield]: https://docs.rs/binroots/latest/binroots/field/struct.BinrootsField.html
/// [brsave]: https://docs.rs/binroots/latest/binroots/save/trait.Save.html
/// [brrt]: https://docs.rs/binroots/latest/binroots/save/struct.RootType.html
#[proc_macro_attribute]
pub fn binroots_struct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;
    let vis = &input.vis;

    let mut root_type =
        quote!(const ROOT_TYPE: binroots::save::RootType = binroots::save::RootType::InMemory);

    for a in attr {
        if a.to_string() == "persistent" {
            root_type = quote!(const ROOT_TYPE: binroots::save::RootType = binroots::save::RootType::Persistent);
        }
    }

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = input.data
    {
        &fields.named
    } else {
        panic!("#[binroots_struct] only supports named struct fields")
    };

    let field_names = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = &field.ident.as_ref().unwrap().to_string();
        let field_type = &field.ty;

        quote! {
            #field_name: binroots::field::BinrootsField<#field_name_str, #field_type>,
        }
    });

    let field_initializers_new = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = &field.ident.as_ref().unwrap().to_string();
        let field_type = &field.ty;

        quote! {
            #field_name: binroots::field::BinrootsField::<#field_name_str, #field_type>::new(#field_name),
        }
    });

    let field_initializers_default = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = &field.ident.as_ref().unwrap().to_string();
        let field_type = &field.ty;

        quote! {
            #field_name: binroots::field::BinrootsField::<#field_name_str, #field_type>::default(),
        }
    });

    let struct_name_str = struct_name.to_string().to_case(Case::Kebab);

    let output = quote! {
        #[derive(Debug, binroots::Serialize)]
        #vis struct #struct_name {
            #( #field_names )*
        }

        impl #struct_name {
            const ROOT_FOLDER: &'static str = #struct_name_str;
            #root_type;
            pub fn new(#fields) -> Self {
                Self {
                    #( #field_initializers_new )*
                }
            }

            pub fn save(&self) -> Result<(), binroots::save::SaveError> {
                binroots::save::Save::save(self, Self::ROOT_FOLDER, Self::ROOT_TYPE)
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #( #field_initializers_default )*
                }
            }
        }

    };

    output.into()
}
