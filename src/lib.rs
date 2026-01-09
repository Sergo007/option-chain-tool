use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};

/// A procedural macro for safe optional chaining in Rust.
///
/// The `opt!` macro provides a concise syntax for chaining operations on `Option` and `Result` types,
/// similar to optional chaining in languages like TypeScript or Swift. It automatically handles
/// unwrapping and propagates `None` values through the chain.
///
/// # Syntax
///
/// The macro supports several operators for different use cases:
///
/// - `?.` - Unwraps an `Option`, returns `None` if the value is `None`
/// - `?Ok.` - Unwraps a `Result` to its `Ok` variant, returns `None` if `Err`
/// - `?Err.` - Unwraps a `Result` to its `Err` variant, returns `None` if `Ok`
/// - `.field` - Access a field without unwrapping (for required fields)
///
/// The macro returns `Some(value)` if all operations succeed, or `None` if any step fails.
///
/// # Examples
///
/// ## Basic Option chaining
///
/// ```ignore
/// use option_chain_tool::opt;
///
/// struct User {
///     profile: Option<Profile>,
/// }
///
/// struct Profile {
///     address: Option<Address>,
/// }
///
/// struct Address {
///     city: Option<String>,
/// }
///
/// let user = User {
///     profile: Some(Profile {
///         address: Some(Address {
///             city: Some("New York".to_string()),
///         }),
///     }),
/// };
///
/// // Instead of: user.profile.as_ref().and_then(|p| p.address.as_ref()).and_then(|a| a.city.as_ref())
/// let city: Option<&String> = opt!(user.profile?.address?.city?);
/// assert_eq!(city, Some(&"New York".to_string()));
/// ```
///
/// ## Chaining with method calls
///
/// ```ignore
/// use option_chain_tool::opt;
///
/// impl Address {
///     fn get_city(&self) -> Option<&String> {
///         self.city.as_ref()
///     }
/// }
///
/// let city: Option<&String> = opt!(user.profile?.address?.get_city()?);
/// ```
///
/// ## Accessing required fields
///
/// ```ignore
/// use option_chain_tool::opt;
///
/// struct Address {
///     city: Option<String>,
///     street: String, // Required field
/// }
///
/// // Access a required field in the chain (no ? after street)
/// let street: Option<&String> = opt!(user.profile?.address?.street);
/// ```
///
/// ## Working with Result types
///
/// ```ignore
/// use option_chain_tool::opt;
///
/// struct Address {
///     validation: Result<String, String>,
/// }
///
/// // Extract the Ok variant
/// let ok_value: Option<&String> = opt!(user.profile?.address?.validation?Ok);
///
/// // Extract the Err variant
/// let err_value: Option<&String> = opt!(user.profile?.address?.validation?Err);
/// ```
///
/// ## Complex chaining
///
/// ```ignore
/// use option_chain_tool::opt;
///
/// // Combine multiple patterns in a single chain
/// let value: Option<&String> = opt!(
///     user
///         .profile?        // Unwrap Option<Profile>
///         .address?        // Unwrap Option<Address>
///         .street          // Access required field
///         .validation?Ok   // Unwrap Result to Ok variant
/// );
/// ```
///
/// # Returns
///
/// - `Some(value)` if all operations in the chain succeed
/// - `None` if any operation in the chain returns `None` or encounters an unwrappable value
///
/// # Notes
///
/// The macro generates nested `if let` expressions that short-circuit on `None`, providing
/// efficient and safe optional chaining without runtime panics.
#[proc_macro]
pub fn opt(input: TokenStream) -> TokenStream {
    let resp = split_on_optional_variants(input);
    // for r in resp.iter() {
    //     let tokens = r
    //         .tokens
    //         .clone()
    //         .into_iter()
    //         .collect::<TokenStream>()
    //         .to_string();
    //     dbg!(format!("Variant: {:?}, Tokens: {}", r.variant, tokens));
    // }
    // dbg!(resp.len());
    let mut result = TokenStream::new();
    let segments_len = resp.len();
    for (index, segment) in resp.into_iter().rev().enumerate() {
        if segments_len - 1 == index {
            if result.is_empty() {
                let mut ____v = TokenStream::new();
                ____v.extend([TokenTree::Ident(Ident::new(
                    "____v",
                    proc_macro::Span::call_site(),
                ))]);
                result = some_wrapper(____v);
            }
            result = if_let(
                segment.variant,
                segment.tokens.into_iter().collect(),
                result,
                true,
            );
            continue;
        }
        {
            let mut is_add_amp = true;
            if index == 0 {
                if ends_with_fn_call(&segment.tokens) {
                    is_add_amp = false;
                }
            }

            let mut after_eq = TokenStream::new();
            after_eq.extend([
                TokenTree::Ident(Ident::new("____v", proc_macro::Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Joint)),
            ]);
            after_eq.extend(segment.tokens.into_iter());
            if result.is_empty() {
                let mut ____v = TokenStream::new();
                ____v.extend([TokenTree::Ident(Ident::new(
                    "____v",
                    proc_macro::Span::call_site(),
                ))]);
                result = some_wrapper(____v);
            }
            result = if_let(segment.variant, after_eq, result, is_add_amp);
        }
    }

    result
}

fn some_wrapper(body: TokenStream) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Ident(Ident::new(
        "Some",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, body))]);
    ts
}

fn ends_with_fn_call(tokens: &[TokenTree]) -> bool {
    let last = match tokens.last() {
        Some(tt) => tt,
        None => return false,
    };

    if let TokenTree::Group(group) = last {
        if group.delimiter() == Delimiter::Parenthesis {
            return true;
        }
    }

    false
}

fn if_let(
    variant: OptionalVariant,
    after_eq: TokenStream,
    body: TokenStream,
    is_add_amp: bool,
) -> TokenStream {
    let mut ts = TokenStream::new();
    ts.extend([TokenTree::Ident(Ident::new(
        "if",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Ident(Ident::new(
        "let",
        proc_macro::Span::call_site(),
    ))]);
    match variant {
        OptionalVariant::Option => {
            ts.extend([TokenTree::Ident(Ident::new(
                "Some",
                proc_macro::Span::call_site(),
            ))]);
        }
        OptionalVariant::Ok => {
            ts.extend([TokenTree::Ident(Ident::new(
                "Ok",
                proc_macro::Span::call_site(),
            ))]);
        }
        OptionalVariant::Err => {
            ts.extend([TokenTree::Ident(Ident::new(
                "Err",
                proc_macro::Span::call_site(),
            ))]);
        }
        OptionalVariant::Required => {
            // panic!("if_let called with Required variant");
        }
        OptionalVariant::Root => {
            panic!("if_let called with Root variant");
        }
    }
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        TokenTree::Ident(Ident::new("____v", proc_macro::Span::call_site())).into(),
    ))]);
    ts.extend([TokenTree::Punct(Punct::new('=', Spacing::Alone))]);
    if is_add_amp {
        ts.extend([TokenTree::Punct(Punct::new('&', Spacing::Joint))]);
    }
    ts.extend(after_eq);
    ts.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
    ts.extend([TokenTree::Ident(Ident::new(
        "else",
        proc_macro::Span::call_site(),
    ))]);
    ts.extend([TokenTree::Group(Group::new(
        Delimiter::Brace,
        TokenTree::Ident(Ident::new("None", proc_macro::Span::call_site())).into(),
    ))]);
    ts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OptionalVariant {
    Root,     // first segment (no ?)
    Option,   // ?.
    Ok,       // ?Ok.
    Err,      // ?Err.
    Required, // no ?
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
                        iter.next(); // consume '.'
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
            }

            _ => current.push(tt.clone()),
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
    let result_len = result.len();
    match input_tokens.last().unwrap() {
        TokenTree::Punct(p) if p.as_char() == '?' => {
            result[result_len - 1].variant = OptionalVariant::Option;
        }
        TokenTree::Ident(p) if p.to_string() == "Ok" => {
            result[result_len - 1].variant = OptionalVariant::Ok;
        }
        TokenTree::Ident(p) if p.to_string() == "Err" => {
            result[result_len - 1].variant = OptionalVariant::Err;
        }
        _ => {
            result[result_len - 1].variant = OptionalVariant::Required;
        }
    }
    result
}
