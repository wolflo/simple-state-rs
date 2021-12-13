use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn ethtest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() { panic!("Non async test fn."); }
    // TODO: check that return type is Result<()>

    let state_on = match input.sig.inputs.first().unwrap() {
        syn::FnArg::Typed(pat) => match &*pat.ty {
            syn::Type::Path(path) => &path.path.segments.first().unwrap().ident,
            _ => panic!(""),
        },
        _ => panic!(""),
    };
    let state_big = state_on.to_string().to_uppercase();

    let name = &input.sig.ident;

    println!("name: {:?}", name);
    println!("state_big: {}{}", "TESTS_ON_", state_big);

    let tests_id = format_ident!("TESTS_ON_{}", state_big);

    let res = quote! {
        const _: () = {
            #[distributed_slice(#tests_id)]
            static __: Test<#state_on> = Test { name: stringify!(#name), run: |s| Box::pin(#name(s)) };
        };
        #input
    };
    res.into()
}
