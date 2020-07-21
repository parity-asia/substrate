use super::{take_item_attrs, CheckTypeDefOptionalGenerics, get_doc_literals};
use quote::ToTokens;
use syn::spanned::Spanned;

/// List of additional token to be used for parsing.
mod keyword {
	syn::custom_keyword!(metadata);
}

pub struct EventDef {
	pub item: syn::ItemEnum,
	/// Event metadatas: `(name, args, docs)`.
	pub metadata: Vec<(syn::Ident, Vec<syn::Ident>, Vec<syn::Lit>)>,
	/// Weither event is declared with instance,
	/// if trait is instantiable and event is generic then event must be declared with instance.
	pub has_instance: bool,
	pub is_generic: bool,
}

impl EventDef {
	pub fn event_use_gen(&self) -> proc_macro2::TokenStream {
		if self.is_generic {
			if self.has_instance {
				quote::quote!(T, I)
			} else {
				quote::quote!(T)
			}
		} else {
			quote::quote!()
		}
	}

	pub fn event_impl_block_gen(&self) -> proc_macro2::TokenStream {
		if self.is_generic {
			if self.has_instance {
				quote::quote!(T: Trait<I>, I: Instance)
			} else {
				quote::quote!(T: Trait)
			}
		} else {
			quote::quote!()
		}
	}
}

impl EventDef {
	pub fn try_from(item: syn::Item) -> syn::Result<Self> {
		if let syn::Item::Enum(mut item) = item {
			let mut event_attrs: Vec<PalletEventAttr> = take_item_attrs(&mut item.attrs)?;
			if event_attrs.len() > 1 {
				let msg = "Invalid pallet::metadata, expected only one attribute \
					`pallet::metadata`";
				return Err(syn::Error::new(event_attrs[1].span, msg));
			}
			let metadata = event_attrs.pop().map_or(vec![], |attr| attr.metadata);

			if !matches!(item.vis, syn::Visibility::Public(_)) {
				let msg = "Invalid pallet::event, `Error` must be public";
				return Err(syn::Error::new(item.span(), msg));
			}
			if item.generics.where_clause.is_some() {
				let msg = "Invalid pallet::event, unexpected where clause";
				return Err(syn::Error::new(item.generics.where_clause.unwrap().span(), msg));
			}

			syn::parse2::<CheckTypeDefOptionalGenerics>(item.generics.params.to_token_stream())?;
			let has_instance = item.generics.params.len() == 2;
			let is_generic = item.generics.params.len() > 0;

			let metadata = item.variants.iter()
				.map(|variant| {
					let name = variant.ident.clone();
					let docs = get_doc_literals(&variant.attrs);
					let args = variant.fields.iter()
						.map(|field| {
							metadata.iter().find(|m| m.0 == field.ty)
								.map(|m| m.1.clone())
								.or_else(|| {
									if let syn::Type::Path(p) = &field.ty {
										p.path.segments.last().map(|s| s.ident.clone())
									} else {
										None
									}
								})
								.ok_or_else(|| {
									let msg = "Invalid pallet::event, type can't be parsed for \
										metadata, must be either a path type (and thus last \
										segments ident is metadata) or match a type in the \
										metadata attributes";
									syn::Error::new(field.span(), msg)
								})
						})
						.collect::<syn::Result<_>>()?;

					Ok((name, args, docs))
				})
				.collect::<syn::Result<_>>()?;

			Ok(EventDef {
				item,
				metadata,
				has_instance,
				is_generic,
			})
		} else {
			Err(syn::Error::new(item.span(), "Invalid pallet::event, expect item enum"))
		}
	}
}

pub struct PalletEventAttr {
	metadata: Vec<(syn::Type, syn::Ident)>,
	span: proc_macro2::Span,
}

fn parse_event_metadata_element(input: syn::parse::ParseStream) -> syn::Result<(syn::Type, syn::Ident)> {
	let typ = input.parse::<syn::Type>()?;
	input.parse::<syn::Token![=]>()?;
	let ident = input.parse::<syn::Ident>()?;
	Ok((typ, ident))
}

impl syn::parse::Parse for PalletEventAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		input.parse::<syn::Token![#]>()?;
		let content;
		syn::bracketed!(content in input);
		content.parse::<syn::Ident>()?;
		content.parse::<syn::Token![::]>()?;

		let lookahead = content.lookahead1();
		if lookahead.peek(keyword::metadata) {
			let span = content.parse::<keyword::metadata>()?.span();
			let metadata_content;
			syn::parenthesized!(metadata_content in content);

			let metadata = metadata_content
				.parse_terminated::<_, syn::Token![,]>(parse_event_metadata_element)?
				.into_pairs()
				.map(syn::punctuated::Pair::into_value)
				.collect();

			Ok(PalletEventAttr { metadata, span })
		} else {
			Err(lookahead.error())
		}
	}
}
