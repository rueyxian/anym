#![no_std]

use core::fmt::Write;

use numtoa::NumToA;
use quote::quote;
use syn::braced;
use syn::parenthesized;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::token::Paren;
use syn::Token;

#[proc_macro]
pub fn anym(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Anym).0.into()
}

struct Anym(proc_macro2::TokenStream);

impl syn::parse::Parse for Anym {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let m = if input.peek(syn::Ident) {
            (input.peek2(Paren), input.peek2(Brace))
        } else {
            (input.peek(Paren), input.peek(Brace))
        };
        let tts = match m {
            (false, false) => parse_unit_struct(input)?,
            (true, false) => parse_tuple_struct(input)?,
            (false, true) => parse_c_struct(input)?,
            _ => unreachable!(
                "It would be insane if `syn::token::Paren` and `syn::token::Brace` coexist"
            ),
        };
        Ok(Anym(tts))
    }
}

fn parse_struct_name(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let nm = if input.peek(syn::Ident) {
        let nm = input.parse::<syn::Ident>()?;
        quote! { #nm }
    } else {
        let nm = gen_struct_nm();
        quote! { #nm }
    };
    Ok(nm)
}

fn parse_unit_struct(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let nm = parse_struct_name(input)?;
    let tts = quote! {{
        struct #nm;
        #nm
    }};
    Ok(tts)
}

fn parse_tuple_struct(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let nm = parse_struct_name(input)?;

    let exprs = {
        let content;
        parenthesized!(content in input);
        Punctuated::<syn::Expr, Token![,]>::parse_terminated(&content)?
    };

    if exprs.is_empty() {
        return Err(syn::Error::new_spanned(
            exprs,
            "Expect at least one argument",
        ));
    }

    let struct_tts = {
        let anots = {
            let tys = (0..exprs.len()).map(gen_ty_anot);
            quote! { <#(#tys,)*> }
        };
        let fields = {
            let tys = (0..exprs.len()).map(gen_ty_anot);
            quote! { (#(#tys,)*) }
        };
        quote! { struct #nm #anots #fields; }
    };

    let bind_tts = {
        let vals = exprs.iter();
        quote! { #nm (#(#vals,)*) }
    };

    let tts = quote! {{
        #struct_tts
        #bind_tts
    }};
    Ok(tts)
}

fn parse_c_struct(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let nm = parse_struct_name(input)?;

    let args = {
        struct Arg {
            ident: proc_macro2::Ident,
            expr: Option<proc_macro2::TokenStream>,
        }
        impl syn::parse::Parse for Arg {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let arg = if input.peek2(Token![:]) {
                    let ident = input.parse::<syn::Ident>()?;
                    let _colon = input.parse::<Token![:]>()?;
                    let expr = {
                        let expr = input.parse::<syn::Expr>()?;
                        quote! { #expr }
                    };
                    Arg {
                        ident,
                        expr: Some(expr),
                    }
                } else {
                    let ident = input.parse::<syn::Ident>()?;
                    Arg { ident, expr: None }
                };
                Ok(arg)
            }
        }
        let content;
        braced!(content in input);
        Punctuated::<Arg, Token![,]>::parse_terminated(&content)?
    };

    if args.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Expect at least one argument",
        ));
    }

    let struct_tts = {
        let anots = {
            let tys = (0..args.len()).map(gen_ty_anot);
            quote! { <#(#tys,)*> }
        };
        let fields = {
            let tys = (0..args.len()).map(gen_ty_anot);
            args.iter().zip(tys).map(|(arg, ty)| {
                let nm = &arg.ident;
                quote! { #nm: #ty }
            })
        };
        quote! {
            struct #nm #anots {
                #(#fields,)*
            }
        }
    };

    let bind_tts = {
        let binds = args.iter().map(|arg| {
            let nm = &arg.ident;
            if let Some(expr) = &arg.expr {
                quote! { #nm: #expr }
            } else {
                quote! { #nm }
            }
        });
        quote! { #nm { #(#binds,)* } }
    };

    let tts = quote! {{
        #struct_tts
        #bind_tts
    }};
    Ok(tts)
}

fn gen_struct_nm() -> proc_macro2::Ident {
    const ID_CHAR_SET: [char; 62] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    let id = nanoid::nanoid!(8, &ID_CHAR_SET);
    let mut buf = [0_u8; 15];
    let mut w = BufW::new(&mut buf);
    w.write_str("__Anym_").unwrap();
    w.write_str(&id).unwrap();
    proc_macro2::Ident::new(w.utf8(), proc_macro2::Span::call_site())
}

fn gen_ty_anot(n: usize) -> proc_macro2::Ident {
    let mut bufn = [0u8; 20];
    let n_str = n.numtoa_str(10, &mut bufn);
    let mut buf = [0_u8; 20];
    let mut w = BufW::new(&mut buf);
    w.write_char('T').unwrap();
    w.write_str(n_str).unwrap();
    proc_macro2::Ident::new(w.utf8(), proc_macro2::Span::call_site())
}

struct BufW<'a> {
    buf: &'a mut [u8],
    idx: usize,
}

impl<'a> BufW<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        assert!(!buf.is_empty());
        BufW { buf, idx: 0 }
    }
    fn utf8(&self) -> &str {
        core::str::from_utf8(&self.buf[0..self.idx]).unwrap()
    }
}

impl<'a> core::fmt::Write for BufW<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        assert!(s.is_ascii());
        let src = s.as_bytes();
        let remain = &mut self.buf[self.idx..];
        if remain.len() < src.len() {
            return Err(core::fmt::Error);
        }
        let tgt = &mut remain[..src.len()];
        tgt.copy_from_slice(src);
        self.idx += src.len();
        Ok(())
    }
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if self.buf.len() < self.idx {
            return Err(core::fmt::Error);
        }
        self.buf[self.idx] = c as u8;
        self.idx += 1;
        Ok(())
    }
}
