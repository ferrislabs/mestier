use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, ItemFn, ItemStruct, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Tags a repository struct with its `(domain, backend)` pair.
///
/// Expands to a [`RepoFor<Domain>`] impl on the backend marker, which lets
/// `#[transactional]` resolve the concrete repo type without name-parsing.
///
/// # Requirements
/// - The struct has a `pub fn new(tx: &SharedTx<'tx>) -> Self`.
/// - Domain and backend marker zero-sized types exist under
///   `mestier_core::infrastructure::registry::{domain, backend}`.
///
/// # Example
/// ```ignore
/// #[repository(domain = Organization, backend = Postgres)]
/// pub struct PgOrganizationRepository<'tx> { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn repository(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as RepositoryAttrs);
    let item = parse_macro_input!(input as ItemStruct);

    let struct_name = &item.ident;
    let domain = &attrs.domain;
    let backend = &attrs.backend;

    let expanded = quote! {
        #item

        impl ::mestier_core::infrastructure::registry::RepoFor<
            ::mestier_core::infrastructure::registry::domain::#domain
        > for ::mestier_core::infrastructure::registry::backend::#backend {
            type Repo<'tx> = #struct_name<'tx>;

            fn build<'tx>(
                tx: &::mestier_core::infrastructure::postgres::SharedTx<'tx>,
            ) -> Self::Repo<'tx> {
                #struct_name::new(tx)
            }
        }
    };

    expanded.into()
}

struct RepositoryAttrs {
    domain: Ident,
    backend: Ident,
}

impl Parse for RepositoryAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut domain = None;
        let mut backend = None;

        let entries = Punctuated::<KeyValue, Token![,]>::parse_terminated(input)?;
        for KeyValue { key, value } in entries {
            match key.to_string().as_str() {
                "domain" => domain = Some(value),
                "backend" => backend = Some(value),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{other}`, expected `domain` or `backend`"),
                    ));
                }
            }
        }

        Ok(Self {
            domain: domain
                .ok_or_else(|| syn::Error::new(input.span(), "missing required key `domain`"))?,
            backend: backend
                .ok_or_else(|| syn::Error::new(input.span(), "missing required key `backend`"))?,
        })
    }
}

struct KeyValue {
    key: Ident,
    value: Ident,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Ident>()?;
        Ok(Self { key, value })
    }
}

/// Wraps an async method body in a SQL transaction, optionally instantiating
/// repositories for the domains you list.
///
/// Each domain name (snake_case) is pascal-cased, looked up under
/// `registry::domain::*`, and resolved via the active [`Backend`] alias to its
/// concrete repository. The resulting binding is named `{name}_repository`.
///
/// Two special names are reserved and do not refer to repositories:
///
/// - `authz` emits `let authz = &self.authz;` (the use-case's [`Authorizer`]),
///   which callers pass to services generic over `A: Authorizer`.
/// - `events` emits a `RealtimeEventPublisher` built from `self.hub`, scoped to
///   this transaction. Building it here rather than holding one on the use case
///   is deliberate: a publisher shared between requests means one buffer for
///   the whole process, and the first transaction to flush drains everybody
///   else's events onto its own organization's channel.
///
/// # Conventions
/// - `self` must have a `pool: PgPool` field.
/// - When `authz` is listed, `self` must also have an `authz` field whose
///   type implements [`authz::Authorizer`].
/// - When `events` is listed, `self` must also have a `hub: EventHub` field.
/// - `self` must have an `actor: events::Actor` field: the durable emitter is
///   built for every transaction, whether or not `emitter` is listed.
/// - Each listed domain (other than `authz`) must have a registered impl
///   (see `#[repository]`).
///
/// # Examples
/// ```ignore
/// #[transactional(organization, role, member, authz)]
/// pub async fn update_organization(&self, cmd: ...) -> Result<...> {
///     let mut service = OrganizationService::new(
///         organization_repository, role_repository, member_repository, authz,
///     );
///     service.update_organization(cmd).await
/// }
///
/// #[transactional]
/// pub async fn raw(&self) -> Result<(), CoreError> { /* use `tx` */ Ok(()) }
/// ```
#[proc_macro_attribute]
pub fn transactional(args: TokenStream, input: TokenStream) -> TokenStream {
    let repos = parse_macro_input!(args as DomainList);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);

    let bindings = repos.names.iter().map(|name| {
        if name == "authz" {
            // Non-repository binding: expose `&self.authz` under the name
            // `authz` for the wrapped body.
            return quote! { let authz = &self.authz; };
        }
        if name == "emitter" {
            // The durable emitter is always created (see below); listing it
            // only asks the macro to name it for the body.
            return quote! { let emitter = &__event_emitter; };
        }
        if name == "events" {
            // Non-repository binding: a publisher scoped to *this* transaction.
            // It is built here, and nowhere else, so no two transactions can
            // ever share a buffer — which is how events used to be delivered
            // to the wrong organization. It writes into a buffer created
            // *outside* the closure, so the buffer survives to be flushed after
            // the commit rather than during the transaction.
            return quote! {
                let events = ::mestier_core::infrastructure::realtime::RealtimeEventPublisher::new(
                    __realtime_buffer.clone(),
                );
            };
        }
        let binding = Ident::new(&format!("{name}_repository"), name.span());
        let domain_ty = Ident::new(&pascal_case(&name.to_string()), name.span());
        quote! {
            let #binding = <
                ::mestier_core::infrastructure::registry::Backend
                as ::mestier_core::infrastructure::registry::RepoFor<
                    ::mestier_core::infrastructure::registry::domain::#domain_ty
                >
            >::build(&tx);
        }
    });

    let body_stmts = &block.stmts;

    // The realtime buffer only exists when `events` is listed: `self.hub` is
    // only required of types that ask for a publisher.
    let emits_realtime = repos.names.iter().any(|name| name == "events");

    let realtime_setup = if emits_realtime {
        quote! {
            let __realtime_buffer =
                ::mestier_core::infrastructure::realtime::RealtimeBuffer::new();
        }
    } else {
        quote! {}
    };

    // Flushing here, on the success arm only, is what makes the broadcast
    // conditional on the commit. Emitting from inside the closure — as the 21
    // hand-written call sites used to — published events for transactions that
    // could still fail at `append` or at `commit`.
    let realtime_flush = if emits_realtime {
        quote! {
            if __tx_outcome.is_ok() {
                __realtime_buffer.flush(&self.hub);
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            // Built for every transaction, listed or not: persistence must not
            // depend on a module remembering to ask for it. A body that emits
            // nothing drains an empty buffer and costs no query.
            let __event_emitter =
                ::mestier_core::infrastructure::automation::TransactionalEventEmitter::new(
                    self.actor,
                    None,
                );
            #realtime_setup
            let __tx_outcome = ::mestier_core::infrastructure::postgres::with_tx_emitting(
                &self.pool,
                &__event_emitter,
                async |tx| {
                    #(#bindings)*
                    #(#body_stmts)*
                },
            ).await;
            #realtime_flush
            __tx_outcome
        }
    };

    expanded.into()
}

struct DomainList {
    names: Vec<Ident>,
}

impl Parse for DomainList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { names: Vec::new() });
        }
        let punctuated = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            names: punctuated.into_iter().collect(),
        })
    }
}

fn pascal_case(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    let mut upper_next = true;
    for c in snake.chars() {
        if c == '_' {
            upper_next = true;
        } else if upper_next {
            out.extend(c.to_uppercase());
            upper_next = false;
        } else {
            out.push(c);
        }
    }
    out
}
