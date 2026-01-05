use std::str::FromStr;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};

/// A procedural macro that splits an optional chaining expression into its segments.
///
/// foo!(test_struct.value?.get((|| 1)())?.value?.value)
///
/// (|| test_struct.value.as_ref()?.get((|| 1)())?.value.as_ref()?.value.as_ref())().copied()
#[proc_macro]
pub fn opt(input: TokenStream) -> TokenStream {
    // dbg!(input.to_string());

    // let aa = TokenStream::from_str("test_struct.value").unwrap();
    // let resp = if_let_some(
    //     aa,
    //     TokenStream::from_str(
    //         "{
    //         // body
    //     }",
    //     )
    //     .unwrap(),
    // );

    // dbg!(resp.to_string());
    let aa = TokenStream::from_str("test_struct.value?Ok.my.vec?.my_val.get(0)?.some_field?.ok()?Err")
        .unwrap();
    // dbg!(aa.clone());
    let resp = split_on_optional_variants(aa);
    for r in resp.iter() {
        dbg!(
            &r.variant,
            r.tokens
                .clone()
                .into_iter()
                .collect::<TokenStream>()
                .to_string()
        );
    }
    dbg!(resp.len());

    //
    let split_tokens = split_on_optional_chain(input);
    let split_tokens_len = split_tokens.len();
    let as_ref = TokenStream::from_str(".as_ref()?.").unwrap();
    let mut expr = TokenStream::new();
    let mut is_last_optional_chain = false;
    for (i, segment) in split_tokens.into_iter().enumerate() {
        let segment_len = segment.len();
        for (i_tt, tt) in segment.into_iter().enumerate() {
            // Skip the last '?' in the segment
            let is_question_mark = match &tt {
                TokenTree::Punct(p) if p.as_char() == '?' => true,
                _ => false,
            };
            if is_question_mark && i_tt == segment_len - 1 {
                is_last_optional_chain = true;
                // dbg!(segment_len, i_tt, i);
                continue;
            }
            expr.extend(TokenStream::from(tt));
        }
        if i != split_tokens_len - 1 {
            expr.extend(as_ref.clone());
        }
    }
    if is_last_optional_chain {
        expr.extend(TokenStream::from_str(".as_ref()?").unwrap());
    }
    let expr = wrap_some(expr);
    let mut clogure = TokenStream::from_str("|| ").unwrap();
    clogure.extend(expr);
    let resp = call_existing_closure(clogure);
    // dbg!(resp.clone());
    // dbg!(resp.to_string());
    resp
}

fn wrap_some(expr: TokenStream) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Ident(Ident::new(
        "Some",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, expr))]);
    ts
}

/// Calls an existing closure represented by the TokenStream.
/// closure: TokenStream ==> represents a closure like `|| expr`
/// responds to TokenStream representing
/// (|| expr )()
fn call_existing_closure(closure: TokenStream) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        closure,
    ))]);
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        TokenStream::new(),
    ))]);
    ts
}

fn split_on_optional_chain(input: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut iter = input.into_iter().peekable();

    let mut segments: Vec<Vec<TokenTree>> = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();

    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == '?' && p.spacing() == Spacing::Joint => {
                if let Some(TokenTree::Punct(dot)) = iter.peek() {
                    if dot.as_char() == '.' && dot.spacing() == Spacing::Alone {
                        // Finish current segment
                        if !current.is_empty() {
                            segments.push(std::mem::take(&mut current));
                        }

                        // Consume the '.'
                        iter.next();
                        continue;
                    }
                }

                // Not actually '?.' → keep '?'
                current.push(tt);
            }
            _ => current.push(tt),
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn if_let_some(after_eq: TokenStream, body: TokenStream) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Ident(Ident::new(
        "if",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Ident(Ident::new(
        "let",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Ident(Ident::new(
        "Some",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        TokenTree::Ident(Ident::new("____v", proc_macro::Span::call_site())).into(),
    ))]);
    ts.extend([TokenTree::Punct(Punct::new('=', Spacing::Alone))]);
    ts.extend([TokenTree::Punct(Punct::new('&', Spacing::Joint))]);
    ts.extend(after_eq);
    ts.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
    ts.extend([TokenTree::Ident(Ident::new(
        "else",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Brace,
        TokenStream::from_str("None").unwrap(),
    ))]);
    ts
}

// use proc_macro::{Ident, Punct, Spacing, TokenStream, TokenTree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OptionalVariant {
    Root,   // first segment (no ?)
    Option, // ?.
    Ok,     // ?Ok.
    Err,    // ?Err.
}

#[derive(Debug, Clone)]
pub(crate) struct OptionalSegment {
    pub variant: OptionalVariant,
    pub tokens: Vec<TokenTree>,
}

pub(crate) fn split_on_optional_variants(input: TokenStream) -> Vec<OptionalSegment> {
    let input_tokens: Vec<TokenTree> = input.clone().into_iter().collect();
    let mut iter = input.into_iter().peekable();

    let mut result: Vec<OptionalSegment> = Vec::new();
    let mut current: Vec<TokenTree> = Vec::new();
    let mut current_variant = OptionalVariant::Root;
    while let Some(tt) = iter.next().as_ref() {
        match &tt {
            TokenTree::Punct(q) if q.as_char() == '?' => {
                // Try to detect ?. / ?Ok. / ?Err.
                let variant = match iter.peek() {
                    Some(TokenTree::Punct(dot)) if dot.as_char() == '.' => {
                        iter.next(); // consume '.'                            // consume '.'

                        Some(OptionalVariant::Option)
                    }

                    Some(TokenTree::Ident(ident))
                        if ident.to_string() == "Ok" || ident.to_string() == "Err" =>
                    {
                        let ident = ident.clone();
                        let v = if ident.to_string() == "Ok" {
                            OptionalVariant::Ok
                        } else {
                            OptionalVariant::Err
                        };

                        // consume Ident
                        iter.next();

                        // require trailing '.'
                        match &iter.next() {
                            Some(TokenTree::Punct(dot)) if dot.as_char() == '.' => Some(v),
                            other => {
                                // rollback-ish: treat as normal tokens
                                // current.push(tt.clone());
                                // current.push(TokenTree::Ident(ident));
                                if let Some(o) = other {
                                    current.push(o.clone());
                                }
                                None
                            }
                        }
                    }

                    _ => None,
                };

                if let Some(v) = variant {
                    if !current.is_empty() {
                        result.push(OptionalSegment {
                            variant: current_variant,
                            tokens: std::mem::take(&mut current),
                        });
                    }

                    current_variant = v;
                    continue;
                }

                // Not a recognized optional-chain operator
                // dbg!(tt.to_string());
            }

            _ => {
                // dbg!(tt.to_string());
                current.push(tt.clone())
            }
        }
    }

    result.push(OptionalSegment {
        variant: current_variant,
        tokens: current,
    });

    for i in 0..result.len() - 1 {
        result[i].variant = result[i + 1].variant.clone();
    }

    // dbg!(last_token.to_string());
    if input_tokens.last().is_none() {
        return result;
    }

    match input_tokens.last().unwrap() {
        TokenTree::Punct(p) if p.as_char() == '?' => {
            result.push(OptionalSegment {
                variant: OptionalVariant::Option,
                tokens: vec![],
            });
        }
        TokenTree::Ident(p) if p.to_string() == "Ok" => {
            result.push(OptionalSegment {
                variant: OptionalVariant::Ok,
                tokens: vec![],
            });
        }
        TokenTree::Ident(p) if p.to_string() == "Err" => {
            result.push(OptionalSegment {
                variant: OptionalVariant::Err,
                tokens: vec![],
            });
        }
        _ => {
            // TODO add more detailed message about it
            // end of ?. / ?Ok. / ?Err. is missing
            // unreachable!("Unexpected last token: {}", last_token.to_string());
        }
    }
    result
}
