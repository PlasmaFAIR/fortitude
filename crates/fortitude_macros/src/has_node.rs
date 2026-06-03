use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Error};

/// Derive [`HasNode`] on a struct
pub(crate) fn derive_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        ident,
        data: Data::Struct(DataStruct { .. }),
        generics,
        ..
    } = input
    else {
        return Err(Error::new(input.ident.span(), "only structs are supported"));
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics HasNode<'a> for #ident #ty_generics #where_clause {
            #[inline]
            fn node(&self) -> &Node<'a> {
                &self.node
            }
        }
    })
}
