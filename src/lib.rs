use std::str::FromStr;

use proc_macro::{Delimiter, Group, Ident, Spacing, TokenStream, TokenTree};

/// A procedural macro that splits an optional chaining expression into its segments.
///
/// foo!(test_struct.value?.get((|| 1)())?.value?.value)
///
/// (|| test_struct.value.as_ref()?.get((|| 1)())?.value.as_ref()?.value.as_ref())().copied()
#[proc_macro]
pub fn opt(input: TokenStream) -> TokenStream {
    let (is_last_optional_chain, input) = remove_last_question_mark(input.clone());
    let split_tokens = split_on_optional_chain(input);
    // dbg!(aa.clone());
    // dbg!(input);
    let as_ref = TokenStream::from_str(".as_ref()?.").unwrap();
    let mut expr = TokenStream::new();
    for (i, segment) in split_tokens.clone().into_iter().enumerate() {
        for tt in segment {
            expr.extend(TokenStream::from(tt));
        }
        if i != split_tokens.len() - 1 {
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

fn remove_last_question_mark(input: TokenStream) -> (bool, TokenStream) {
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut is_rm = false;
    // Find the last `?` token from the end
    if let Some(pos) = tokens.iter().rposition(|tt| match tt {
        TokenTree::Punct(p) if p.as_char() == '?' => true,
        _ => false,
    }) {
        if tokens.len() - 1 == pos {
            is_rm = true;
            tokens.remove(pos);
        }
    }

    // Rebuild a new TokenStream
    (is_rm, tokens.into_iter().collect())
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
