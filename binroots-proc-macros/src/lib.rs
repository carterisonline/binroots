use convert_case::{Case, Casing};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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

#[proc_macro_attribute]
pub fn binroots_struct(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;
    let vis = &input.vis;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = input.data
    {
        &fields.named
    } else {
        panic!("#[binroots_struct] only supports named struct fields")
    };

    /*
    let field_wrappers = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let binroots_field_name = Ident::new(
            &format!("Binroots{}", field_name.as_ref().unwrap()),
            field.span(),
        );

        quote! {
            struct #binroots_field_name {
                field_name: &'static str,
                value: #field_type,
            }

            impl #binroots_field_name {
                pub fn new(value: #field_type) -> Self {
                    Self {
                        field_name: stringify!(#field_name),
                        value,
                    }
                }
            }

            impl From<#field_type> for #binroots_field_name {
                fn from(value: #field_type) -> Self {
                    Self::new(value)
                }
            }

            impl From<#binroots_field_name> for #field_type {
                fn from(binroots: #binroots_field_name) -> Self {
                    binroots.value
                }
            }

            impl Default for #binroots_field_name {
                fn default() -> Self {
                    Self::new(Default::default())
                }
            }

            impl std::fmt::Display for #binroots_field_name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self.value)
                }
            }

            impl std::fmt::Debug for #binroots_field_name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self)
                }
            }
        }
    });
    */

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

    /*let field_accessors = fields.iter().map(|field| {
       let field_name = &field.ident;
       quote! {
           pub fn #field_name(&self) -> &binroots::field::BinrootsField</*Binroots#*/field_name> {
               &self.#field_name
           }
       }
    });*/

    let struct_name_str = struct_name.to_string().to_case(Case::Kebab);

    let output = quote! {
        #[derive(Debug, binroots::Serialize)]
        #vis struct #struct_name {
            #( #field_names )*
        }

        //#( #field_wrappers )*

        impl #struct_name {
            const ROOT_FOLDER: &'static str = #struct_name_str;
            pub fn new(#fields) -> Self {
                Self {
                    #( #field_initializers_new )*
                }
            }

            pub fn save(&self) -> Result<(), binroots::save::SaveError> {
                binroots::save::Save::save(self, Self::ROOT_FOLDER)
            }

            //#( #field_accessors )*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #( #field_initializers_default )*
                }
            }
        }

        /*
        impl std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #( write!(f, "{}: {}\n", stringify!(#field_names), self.#field_names) )*
                Ok(())
            }
        }*/
    };

    output.into()
}
