#![doc = include_str!("../README.md")]
use darling::{FromDeriveInput, FromField, ast::Data, util::PathList};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Ident, Type, parse_macro_input,
    visit::{self, Visit},
};

/// Attributes that can be applied to fields of the struct.
#[derive(FromField, Clone)]
#[darling(attributes(companion))]
struct FieldAttrs {
    ident: Option<Ident>,
    ty: Type,
    /// Rename the enum variant for this field.
    rename: Option<String>,
    /// Skip this field from being included in the companion enums.
    #[darling(default)]
    skip: bool,
}

/// Options for the `EnumCompanion` derive macro.
#[derive(FromDeriveInput)]
#[darling(attributes(companion), supports(struct_named))]
struct CompanionOpts {
    ident: Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: Data<(), FieldAttrs>,
    /// The name of the function to get a value from the struct.
    #[darling(default = "default_value_fn")]
    value_fn: String,
    /// The name of the function to update a value in the struct.
    #[darling(default = "default_update_fn")]
    update_fn: String,
    /// The name of the function to get a list of all fields.
    #[darling(default = "default_fields_fn")]
    fields_fn: String,
    /// A list of traits to derive for the field enum.
    #[darling(default)]
    derive_field: PathList,
    /// A list of traits to derive for the value enum.
    #[darling(default)]
    derive_value: PathList,
    /// Serde attributes for the field enum.
    #[darling(default)]
    serde_field: Option<syn::Meta>,
    /// Serde attributes for the value enum.
    #[darling(default)]
    serde_value: Option<syn::Meta>,
}

/// Default name for the `value` function.
fn default_value_fn() -> String {
    "value".to_string()
}

/// Default name for the `update` function.
fn default_update_fn() -> String {
    "update".to_string()
}

/// Default name for the `fields` function.
fn default_fields_fn() -> String {
    "fields".to_string()
}

#[proc_macro_derive(EnumCompanion, attributes(companion))]
pub fn enum_companion_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Parse the macro options from the derive input.
    let opts = match CompanionOpts::from_derive_input(&input) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    // Get the struct's name, visibility, and other options.
    let struct_name = opts.ident;
    let vis = opts.vis;
    let generics = opts.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let value_fn_name = Ident::new(&opts.value_fn, struct_name.span());
    let update_fn_name = Ident::new(&opts.update_fn, struct_name.span());
    let fields_fn_name = Ident::new(&opts.fields_fn, struct_name.span());
    let derive_field = opts.derive_field;
    let derive_value = opts.derive_value;
    let serde_field = &opts.serde_field;
    let serde_field_attr = if let Some(syn::Meta::List(serde_field)) = serde_field {
        // Convert the serde attributes to a token stream.
        let attr_tokens: proc_macro2::TokenStream = serde_field.tokens.clone();
        quote! { #[serde(#attr_tokens)] }
    } else {
        quote! {}
    };
    let serde_value = &opts.serde_value;
    let serde_value_attr = if let Some(syn::Meta::List(serde_value)) = serde_value {
        let attr_tokens: proc_macro2::TokenStream = serde_value.tokens.clone();
        quote! { #[serde(#attr_tokens)] }
    } else {
        quote! {}
    };

    // Get the struct's fields.
    let fields = opts.data.take_struct().unwrap();

    let mut field_idents = Vec::new();
    let mut field_types = Vec::new();
    let mut field_variants = Vec::new();
    let mut from_str_arms = Vec::new();

    // Iterate over the fields and extract the necessary information.
    for field in fields.fields {
        if field.skip {
            continue;
        }

        let ident = field.ident.clone().unwrap();
        let variant_name_str = field
            .rename
            .clone()
            .unwrap_or_else(|| to_pascal_case(&ident.to_string()));
        let variant = Ident::new(&variant_name_str, ident.span());

        let ident_str = ident.to_string();
        let mut patterns = vec![ident_str.clone()];
        if variant_name_str != ident_str {
            patterns.push(variant_name_str);
        }

        from_str_arms.push(quote! {
            #(#patterns)|* => Ok(Self::#variant)
        });

        field_idents.push(ident);
        field_types.push(field.ty);
        field_variants.push(variant);
    }

    // Create the names for the generated enums.
    let field_enum_name = syn::Ident::new(&format!("{struct_name}Field"), struct_name.span());
    let value_enum_name = syn::Ident::new(&format!("{struct_name}Value"), struct_name.span());

    // Prepare the variants for the field enum.
    let field_enum_variants = field_variants.iter();
    let _field_variants_count = field_variants.len();

    // Prepare the variants for the value enum.
    let value_enum_variants = field_variants
        .iter()
        .zip(field_types.iter())
        .map(|(variant, ty)| {
            quote! { #variant(#ty) }
        });

    // Prepare the match arms for the `value` function.
    let value_match_arms =
        field_idents
            .iter()
            .zip(field_variants.iter())
            .map(|(ident, variant)| {
                quote! {
                    #field_enum_name::#variant => #value_enum_name::#variant(self.#ident.clone())
                }
            });

    // Prepare the match arms for the `update` function.
    let update_match_arms =
        field_idents
            .iter()
            .zip(field_variants.iter())
            .map(|(ident, variant)| {
                quote! {
                    #value_enum_name::#variant(value) => self.#ident = value
                }
            });

    let trait_impl = if opts.value_fn == "value"
        && opts.update_fn == "update"
        && opts.fields_fn == "fields"
    {
        quote! {
            impl #impl_generics ::enum_companion::EnumCompanionTrait<#field_enum_name, #value_enum_name #ty_generics> for #struct_name #ty_generics #where_clause {
                fn value(&self, field: #field_enum_name) -> #value_enum_name #ty_generics {
                    self.value(field)
                }

                fn update(&mut self, value: #value_enum_name #ty_generics) {
                    self.update(value)
                }

                fn fields() -> &'static [#field_enum_name] {
                    &#field_enum_name::FIELDS
                }

                fn as_values(&self) -> Vec<#value_enum_name #ty_generics> {
                    self.as_values()
                }
            }
        }
    } else {
        quote! {}
    };

    let mut unique_types = std::collections::HashMap::new();
    for (ty, variant) in field_types.iter().zip(field_variants.iter()) {
        let key = quote!(#ty).to_string();
        unique_types
            .entry(key)
            .or_insert_with(|| (ty.clone(), Vec::new()))
            .1
            .push(variant.clone());
    }

    let generic_param_idents: std::collections::HashSet<String> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(ty) => Some(ty.ident.to_string()),
            _ => None,
        })
        .collect();

    let try_from_impls = unique_types.values().filter_map(|(ty, variants)| {
        if type_contains_generic(ty, &generic_param_idents) {
            return None;
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        Some(quote! {
            impl #impl_generics std::convert::TryFrom<#value_enum_name #ty_generics> for #ty #where_clause {
                type Error = #value_enum_name #ty_generics;

                fn try_from(value: #value_enum_name #ty_generics) -> Result<Self, Self::Error> {
                    match value {
                        #(#value_enum_name::#variants(val) => Ok(val)),*,
                        _ => Err(value),
                    }
                }
            }
        })
    });

    let try_into_impls = unique_types.values().filter_map(|(ty, variants)| {
        if type_contains_generic(ty, &generic_param_idents) {
            return None;
        }

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let arms = variants.iter().map(|variant| {
            quote! {
                #field_enum_name::#variant => Ok(#value_enum_name::#variant(value)),
            }
        });

        Some(quote! {
            impl #impl_generics std::convert::TryFrom<(#field_enum_name, #ty)> for #value_enum_name #ty_generics #where_clause {
                type Error = #field_enum_name;

                fn try_from(value: (#field_enum_name, #ty)) -> Result<Self, Self::Error> {
                    let (field, value) = value;
                    match field {
                        #(#arms)*
                        _ => Err(field),
                    }
                }
            }
        })
    });

    // Generate the final token stream.
    let expanded = quote! {
        /// An enum representing the fields of the struct.
        #[allow(dead_code)]
        #[derive(Copy, Clone, #(#derive_field),*)]
        #serde_field_attr
        #vis enum #field_enum_name {
            #(#field_enum_variants),*
        }

        impl #field_enum_name {
            pub const FIELDS: &'static [#field_enum_name] = &[#(#field_enum_name::#field_variants),*];
        }

        /// An enum representing the values of the struct's fields.
        #[allow(dead_code)]
        #[derive(Clone, #(#derive_value),*)]
        #serde_value_attr
        #vis enum #value_enum_name #ty_generics {
            #(#value_enum_variants),*
        }

        impl std::str::FromStr for #field_enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(format!("Invalid field name: {}", s)),
                }
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            /// Returns an array of all field enum variants.
            pub fn #fields_fn_name() -> &'static [#field_enum_name] {
                #field_enum_name::FIELDS
            }

            /// Returns a vector of all field values.
            pub fn as_values(&self) -> Vec<#value_enum_name #ty_generics> {
                Self::#fields_fn_name()
                    .iter()
                    .map(|&field| self.#value_fn_name(field))
                    .collect()
            }

            /// Returns the value of a specific field.
            pub fn #value_fn_name(&self, field: #field_enum_name) -> #value_enum_name #ty_generics {
                match field {
                    #(#value_match_arms),*
                }
            }

            /// Updates the value of a specific field.
            pub fn #update_fn_name(&mut self, value: #value_enum_name #ty_generics) {
                match value {
                    #(#update_match_arms),*
                }
            }
        }

        #trait_impl

        #(#try_from_impls)*

        #(#try_into_impls)*
    };

    TokenStream::from(expanded)
}

/// Converts a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

struct GenericVisitor<'a> {
    generic_params: &'a std::collections::HashSet<String>,
    contains_generic: bool,
}

impl<'ast, 'a> Visit<'ast> for GenericVisitor<'a> {
    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if self.contains_generic {
            return;
        }
        if i.qself.is_none() {
            if let Some(segment) = i.path.segments.last() {
                if self.generic_params.contains(&segment.ident.to_string()) {
                    self.contains_generic = true;
                    return;
                }
            }
        }
        visit::visit_type_path(self, i);
    }
}

fn type_contains_generic(
    ty: &syn::Type,
    generic_params: &std::collections::HashSet<String>,
) -> bool {
    let mut visitor = GenericVisitor {
        generic_params,
        contains_generic: false,
    };
    visitor.visit_type(ty);
    visitor.contains_generic
}
