use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn ethtest(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

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

// TODO:
// - check that impl is on State
// - get Runner assoc type
// - handle Runner/State assoc types out of order
// - generate impl TestSet
#[proc_macro_attribute]
pub fn ethstate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);

    let state_on = match &*input.self_ty {
        syn::Type::Path(path) => &path.path.segments.first().unwrap().ident,
        _ => panic!(""),
    };
    let state_big = state_on.to_string().to_uppercase();

    let prev_state = match input.items.first().unwrap() {
        syn::ImplItem::Type(ty) => match &ty.ty {
            syn::Type::Path(path) => &path.path.segments.first().unwrap().ident,
            _ => panic!("")
        }
        _ => panic!(""),
    };

    let prev_state_big = prev_state.to_string().to_uppercase();
    let prev_state_id = format_ident!("STATES_FROM_{}", prev_state_big);

    let tests_id = format_ident!("TESTS_ON_{}", state_big);
    let next_states_id = format_ident!("STATES_FROM_{}", state_big);

    // TODO FROM_PREV_STATE
    let res = quote! {
        const _: () = {
            #[distributed_slice(#prev_state_id)]
            static __: StateMove<#prev_state, RunnerType> = |s, r| Box::pin(dispatch::<#prev_state, #state_on, RunnerType>(s, r));
        };

        #[distributed_slice]
        pub static #next_states_id: [StateMove<#state_on, RunnerType>] = [..];

        #[distributed_slice]
        pub static #tests_id: [Test<#state_on>] = [..];

        impl TestSet for #state_on {
            type State = #state_on;
            type Runner = RunnerType;
            fn tests(&self) -> &'static [Test<Self::State>] {
                &#tests_id
            }
            fn children(&self) -> &'static [StateMove<Self::State, Self::Runner>] {
                &#next_states_id
            }
        }
        #input
    };
    res.into()
}
