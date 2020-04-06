#[macro_use]
extern crate quote;

extern crate proc_macro;
use crate::proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use std::result::Result;
use syn::parse::Parser;
use syn::*;

#[proc_macro_attribute]
pub fn v8_ffi(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let parser = punctuated::Punctuated::<Ident, Token![,]>::parse_terminated;
    let ast = parser.parse(metadata).unwrap();
    let inner = ast
        .into_iter()
        .map(|i| format!("{}", i))
        .collect::<Vec<String>>();
    let mut scoped = false;
    for flag in inner {
        if flag == "scoped" {
            scoped = true;
        }
    }
    let ast = parse_macro_input!(input as ItemFn);
    impl_v8_ffi(scoped, &ast)
}

#[proc_macro_hack]
pub fn load_v8_ffi(input: TokenStream) -> TokenStream {
    let parser = punctuated::Punctuated::<Expr, Token![,]>::parse_terminated;
    let ast = parser.parse(input).unwrap();
    let inner = ast.into_iter().collect::<Vec<Expr>>();
    if inner.len() != 3 {
        return quote! {
            compile_error!("invalid call to load_v8_ffi, expected args: ffi function reference, scope, context");
        }.into();
    }
    let function_ref = &inner[0];
    let scope_ref = &inner[1];
    let context_ref = &inner[2];
    let function_ref = match function_ref {
        Expr::Path(ExprPath { path, qself, attrs }) => {
            let mut new_path = path.clone();
            let func_name = new_path.segments.last_mut().unwrap();
            let ffi_ident = Ident::new(
                &format!("__v8_ffi_{}", func_name.ident),
                func_name.ident.span(),
            );
            func_name.ident = ffi_ident;
            Expr::Path(ExprPath {
                path: new_path,
                qself: qself.clone(),
                attrs: attrs.clone(),
            })
        }
        _ => {
            return quote! {
                compile_error!("expected path for ffi function reference");
            }
            .into();
        }
    };
    return quote! { #function_ref(#scope_ref, #context_ref).into() }.into();
}

enum SimpleType {
    This(bool, Path),
    Type(Type),
}

fn parse_simple_type(ty: &Type) -> SimpleType {
    match ty {
        Type::Reference(TypeReference {
            lifetime: None,
            mutability,
            elem,
            ..
        }) => match (mutability, &**elem) {
            (
                mutability,
                Type::Path(TypePath {
                    qself: None,
                    path: x,
                }),
            ) => {
                return SimpleType::This(mutability.is_some(), x.clone());
            }
            _ => {
                return SimpleType::Type(ty.clone());
            }
        },
        _ => {
            return SimpleType::Type(ty.clone());
        }
    }
}

fn impl_v8_ffi(scoped: bool, ast: &ItemFn) -> TokenStream {
    let sig = &ast.sig;
    if sig.constness.is_some() {
        return quote_spanned! {
            sig.constness.unwrap().span =>
            compile_error!("const fn not allowed in v8_ffi");
        }
        .into();
    }
    if sig.asyncness.is_some() {
        return quote_spanned! {
            sig.asyncness.unwrap().span =>
            compile_error!("async fn not allowed in v8_ffi");
        }
        .into();
    }
    if sig.unsafety.is_some() {
        return quote_spanned! {
            sig.unsafety.unwrap().span =>
            compile_error!("unsafe fn not allowed in v8_ffi");
        }
        .into();
    }
    if sig.abi.is_some() {
        return quote_spanned! {
            sig.abi.as_ref().unwrap().extern_token.span =>
            compile_error!("extern fn not allowed in v8_ffi");
        }
        .into();
    }
    if sig.generics.where_clause.is_some() {
        return quote_spanned! {
            sig.generics.where_clause.as_ref().unwrap().where_token.span =>
            compile_error!("generics where clause not allowed in v8_ffi fn");
        }
        .into();
    }
    for param in sig.generics.params.iter() {
        if let GenericParam::Lifetime(_) = param {
            // nop
        } else {
            return quote_spanned! {
                sig.generics.lt_token.as_ref().unwrap().span =>
                compile_error!("non-lifetime generics not allowed in v8_ffi fn");
            }
            .into();
        }
    }
    if sig.variadic.is_some() {
        return quote_spanned! {
            sig.variadic.as_ref().unwrap().dots.spans[0] =>
            compile_error!("variadic not allowed in v8_ffi fn");
        }
        .into();
    }
    let inputs = sig.inputs.iter().collect::<Vec<&FnArg>>();
    for input in &inputs {
        if let FnArg::Receiver(receiver) = input {
            return quote_spanned! {
                receiver.self_token.span =>
                compile_error!("self is not allowed in v8_ffi fn, use `this: &SomeType` as first argument to use auto `ObjectWrap` unwrapping");
            }.into();
        }
    }
    let inputs = inputs
        .iter()
        .map(|x| if let FnArg::Typed(x) = x { x } else { panic!() })
        .collect::<Vec<&PatType>>();
    let inputs: Result<Vec<(Ident, SimpleType)>, _> = inputs
        .into_iter()
        .map(|input| {
            let name = if let Pat::Ident(PatIdent {
                by_ref: None,
                subpat: None,
                ident,
                ..
            }) = &*input.pat
            {
                ident.clone()
            } else {
                return Err(quote_spanned! {
                    input.colon_token.span =>
                    compile_error!("invalid non-ident argument name for v8_ffi fn");
                }
                .into());
            };
            let ty = parse_simple_type(&input.ty);
            Ok((name, ty))
        })
        .collect();
    let mut inputs = match inputs {
        Err(e) => return e,
        Ok(x) => x,
    };
    let this: Vec<(Ident, bool, Path)> = inputs
        .iter()
        .filter_map(|x| {
            if let (name, SimpleType::This(mutability, path)) = x {
                Some((name.clone(), *mutability, path.clone()))
            } else {
                None
            }
        })
        .collect();
    if this.len() > 1 {
        return quote_spanned! {
            sig.fn_token.span =>
            compile_error!("can only object wrap one argument in v8_ffi fn");
        }
        .into();
    }
    let return_type = match &sig.output {
        ReturnType::Default => None,
        ReturnType::Type(arrow, ty) => {
            let return_type = parse_simple_type(&ty);
            if let SimpleType::This(_, _) = &return_type {
                return quote_spanned! {
                    arrow.spans[0] =>
                    compile_error!("cannot return wrapped object from v8_ffi fn");
                }
                .into();
            }
            Some(return_type)
        }
    };
    let this = this.into_iter().next();
    let mut preludes: Vec<TokenStream2> = vec![];

    if let Some((name, mutability, ty)) = &this {
        if name != &inputs[0].0 || format!("{}", name) != "this" {
            return quote_spanned! {
                name.span() =>
                compile_error!("object wrapped argument must be first in v8_ffi fn and be named `this`");
            }.into();
        }
        let ty = Type::Path(TypePath {
            qself: None,
            path: ty.clone(),
        });
        if *mutability {
            preludes.push(quote! {
                let #name: ::std::option::Option<::std::rc::Rc<::std::sync::Mutex<#ty>>> = ::rusty_v8_helper::ObjectWrap::from_object(__v8_ffi_args.this());
                if #name.is_none() {
                    throw_exception(__v8_ffi_scope, "invalid 'this' for ffi call");
                    return;
                }
                let #name = #name.unwrap();
                let mut #name = #name.lock().unwrap();
                let mut #name = &mut #name;
            });
        } else {
            preludes.push(quote! {
                let #name: ::std::option::Option<::std::rc::Rc<#ty>> = ::rusty_v8_helper::ObjectWrap::from_object(__v8_ffi_args.this());
                if #name.is_none() {
                    throw_exception(__v8_ffi_scope, "invalid 'this' for ffi call");
                    return;
                }
                let #name = #name.unwrap();
                let #name = &#name;
            });
        }
        inputs.remove(0);
    }

    if scoped {
        if inputs.len() < 2 {
            return quote_spanned! {
                sig.fn_token.span =>
                compile_error!("scoped function must have at least 2 arguments: scope, context");
            }
            .into();
        }
        let input0_name = format!("{}", &inputs.get(0).as_ref().unwrap().0);
        let input1_name = format!("{}", &inputs.get(1).as_ref().unwrap().0);
        if !(input0_name == "scope" || input0_name == "_scope")
            || !(input1_name == "context" || input1_name == "_context")
        {
            return quote_spanned! {
                sig.fn_token.span =>
                compile_error!("scoped function's first two arguments must be named: scope, context");
            }.into();
        }
        inputs.remove(1);
        inputs.remove(0);
    }

    for (i, input) in inputs.iter().enumerate() {
        let name = &input.0;
        let i = i as i32;
        match &input.1 {
            SimpleType::This(_, _) => {}
            SimpleType::Type(ty) => {
                let from_value_ident = Ident::new("from_value", sig.ident.span());
                let ty = match ty {
                    Type::Path(TypePath { qself, path }) => {
                        let mut path = path.clone();
                        for seg in path.segments.iter_mut() {
                            if let PathArguments::AngleBracketed(args) = &mut seg.arguments {
                                if !args.colon2_token.is_some() {
                                    args.colon2_token = Some(token::Colon2 {
                                        spans: [sig.ident.span(), sig.ident.span()],
                                    });
                                }
                            }
                        }
                        if !path.segments.empty_or_trailing() {
                            path.segments.push_punct(token::Colon2 {
                                spans: [sig.ident.span(), sig.ident.span()],
                            });
                        }
                        path.segments.push_value(PathSegment {
                            ident: from_value_ident,
                            arguments: PathArguments::None,
                        });
                        let ty = Type::Path(TypePath {
                            qself: qself.clone(),
                            path,
                        });
                        quote! { #ty }
                    }
                    _ => quote! { <#ty>::#from_value_ident },
                };
                preludes.push(quote! {
                    let mut #name = __v8_ffi_args.get(#i);
                    let #name = #ty(#name, __v8_ffi_scope, __v8_ffi_context);
                    if let Err(e) = #name {
                        ::rusty_v8_helper::util::throw_exception(__v8_ffi_scope, &format!("{:?}", e));
                        return;
                    }
                    let #name = #name.unwrap();
                })
            }
        }
    }
    let vis = &ast.vis;
    let ffi_internal_ident = Ident::new(
        &format!("__v8_ffi_internal_{}", sig.ident),
        sig.ident.span(),
    );
    let ffi_ident = Ident::new(&format!("__v8_ffi_{}", sig.ident), sig.ident.span());
    let preludes: TokenStream2 = preludes.into_iter().collect();
    let original_ident = &sig.ident;

    let mut arg_names: Vec<TokenStream2> = vec![];
    if this.is_some() {
        let name = &this.as_ref().unwrap().0;
        arg_names.push(quote! { #name, });
    }
    if scoped {
        arg_names.push(quote! { __v8_ffi_scope, });
        arg_names.push(quote! { __v8_ffi_context, });
    }
    for input in inputs.iter() {
        let name = &input.0;
        arg_names.push(quote! { #name, })
    }
    let arg_names: TokenStream2 = arg_names.into_iter().collect();
    let return_postlude = if let Some(SimpleType::Type(_)) = return_type {
        Some(quote! {
            let __v8_ffi_value = __returned.to_value(__v8_ffi_scope, __v8_ffi_context);
            match __v8_ffi_value {
                Ok(__v8_ffi_value) => __v8_ffi_rv.set(__v8_ffi_value),
                Err(e) => {
                    ::rusty_v8_helper::util::throw_exception(__v8_ffi_scope, &format!("{:?}", e));
                    return;
                }
            }

        })
    } else {
        None
    };

    let gen = quote! {
        #ast

        fn #ffi_internal_ident<'sc>(mut __v8_ffi_scope: ::rusty_v8_protryon::FunctionCallbackScope<'sc>, __v8_ffi_args: ::rusty_v8_protryon::FunctionCallbackArguments<'sc>, mut __v8_ffi_rv: ::rusty_v8_protryon::ReturnValue<'sc>) {
            let __v8_ffi_context = __v8_ffi_scope.get_current_context().unwrap();
            #preludes
            let __returned = #original_ident(#arg_names);
            #return_postlude
        }

        #vis fn #ffi_ident<'sc, 'c>(__v8_ffi_scope: &mut impl ::rusty_v8_protryon::ToLocal<'sc>, __v8_ffi_context: ::rusty_v8_protryon::Local<'c, ::rusty_v8_protryon::Context>) -> ::rusty_v8_protryon::Local<'sc, ::rusty_v8_protryon::Function> {
            ::rusty_v8_protryon::Function::new(
                __v8_ffi_scope,
                __v8_ffi_context,
                #ffi_internal_ident,
            ).unwrap()
        }

    };
    gen.into()
}
