extern crate syn;
extern crate quote;
extern crate proc_macro;

use self::proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

use syn::{parse_macro_input, parse_quote, Attribute, ImplItemMethod, Item,
          ItemFn, ItemImpl, ItemMod, ItemTrait, TraitItemMethod, Token};

#[derive(Default)]
struct Timing {
    id_stack: Vec<String>,
}

impl Timing {
    fn push(&mut self, ident: String) {
        self.id_stack.push(ident);
    }

    fn pop(&mut self) {
        let _ = self.id_stack.pop();
    }

    fn name(&self) -> String {
        self.id_stack.join("::")
    }

    fn is_notiming(&self, i: &[Attribute]) -> bool {
        i.iter().any(|ref a| if a.path.segments.len() == 1 {
            let ident = &a.path.segments.iter().next().unwrap().ident;
            ident == "timing" || ident == "notiming"
        } else {
            false
        })
    }
}

impl Parse for Timing {
    fn parse(input: ParseStream) -> Result<Self> {
        let vars = Punctuated::<syn::LitStr, Token![,]>::parse_terminated(input)?;
        Ok(Timing {
            id_stack: vars.into_iter().map(|s| s.value()).collect::<Vec<_>>(),
        })
    }
}

impl Fold for Timing {
    fn fold_item_mod(&mut self, i: ItemMod) -> ItemMod {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        self.push(i.ident.to_string());
        let item_mod = fold::fold_item_mod(self, i);
        self.pop();
        item_mod
    }

    fn fold_item_trait(&mut self, i: ItemTrait) -> ItemTrait {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        self.push(i.ident.to_string());
        let t = fold::fold_item_trait(self, i);
        self.pop();
        t
    }

    fn fold_trait_item_method(&mut self, i: TraitItemMethod)
    -> TraitItemMethod {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        self.push(i.sig.ident.to_string());
        let m = fold::fold_trait_item_method(self, i);
        self.pop();
        m
    }

    fn fold_item_impl(&mut self, i: ItemImpl) -> ItemImpl {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        if let Some((_, ref path, _)) = i.trait_ {
            self.push(format!("{:?} as {:?}", &i.self_ty, path));
        } else {
            self.push(format!("{:?}", &i.self_ty));
        }
        let ii = fold::fold_item_impl(self, i);
        self.pop();
        ii
    }

    fn fold_impl_item_method(&mut self, i: ImplItemMethod) -> ImplItemMethod {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        self.push(i.sig.ident.to_string());
        let method = fold::fold_impl_item_method(self, i);
        self.pop();
        method
    }

    fn fold_item_fn(&mut self, i: ItemFn) -> ItemFn {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        let mut i = i;
        self.push(i.sig.ident.to_string());
        let name = self.name();
        let stmts = ::std::mem::replace(&mut i.block.stmts, vec![parse_quote! {
            let _timing_guard = screeps_timing::start_guard(#name);
        }]);
        for stmt in stmts {
            i.block.stmts.push(fold::fold_stmt(self, stmt));
        }
        self.pop();
        i
    }
}

#[proc_macro_attribute]
pub fn timing(attrs: TokenStream, code: TokenStream) -> TokenStream {
    let input = parse_macro_input!(code as Item);
    let mut timing = parse_macro_input!(attrs as Timing);
    let item = fold::fold_item(&mut timing, input);
    TokenStream::from(quote!(#item))
}

#[proc_macro_attribute]
pub fn notiming(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    code
}