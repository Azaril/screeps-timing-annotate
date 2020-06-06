extern crate syn;
extern crate quote;
extern crate proc_macro;

use self::proc_macro::TokenStream;
use quote::quote;
use syn::Type;
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
        i.iter().any(|ref a| 
            a.path.is_ident("timing") || a.path.is_ident("screeps_timing_annotate::timing") || 
            a.path.is_ident("notiming") || a.path.is_ident("screeps_timing_annotate::notiming")
        )
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

    fn fold_trait_item_method(&mut self, i: TraitItemMethod) -> TraitItemMethod {
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

        let pushed = if let Type::Path(type_path) = i.self_ty.as_ref() {
            let combined_type = type_path.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");

            if let Some((_, ref trait_path, _)) = i.trait_ {
                let combined_trait = trait_path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
    
                self.push(format!("{:?} as {:?}", combined_type, combined_trait));
                
                true
            } else {
                self.push(combined_type);
                true
            }
        } else {
            false
        };

        let ii = fold::fold_item_impl(self, i);

        if pushed {
            self.pop();
        }

        ii
    }

    fn fold_impl_item_method(&mut self, i: ImplItemMethod) -> ImplItemMethod {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        let mut method = fold::fold_impl_item_method(self, i);
        self.push(method.sig.ident.to_string());
        let name = self.name();
        let mut stmts = ::std::mem::replace(&mut method.block.stmts, vec![parse_quote! {
            let _timing_guard = screeps_timing::start_guard(#name);
        }]);
        method.block.stmts.append(&mut stmts);
        self.pop();
        method
    }

    fn fold_item_fn(&mut self, i: ItemFn) -> ItemFn {
        if self.is_notiming(&i.attrs) {
            return i;
        }
        let mut func = fold::fold_item_fn(self, i);
        self.push(func.sig.ident.to_string());
        let name = self.name();
        let mut stmts = ::std::mem::replace(&mut func.block.stmts, vec![parse_quote! {
            let _timing_guard = screeps_timing::start_guard(#name);
        }]);
        func.block.stmts.append(&mut stmts);
        self.pop();
        func
    }
}

#[proc_macro_attribute]
pub fn timing(attrs: TokenStream, code: TokenStream) -> TokenStream {
    let input = parse_macro_input!(code as Item);
    let mut timing = parse_macro_input!(attrs as Timing);
    let item = timing.fold_item(input);
    TokenStream::from(quote!(#item))
}

#[proc_macro_attribute]
pub fn notiming(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    code
}