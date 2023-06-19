#[macro_use]
extern crate quote;
extern crate syn;

mod cparser_lib;

use std::{io::Read, collections::HashMap};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;


#[proc_macro_attribute]
pub fn c_class(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let output = quote! {
        #[repr(C)]
        #[allow(dead_code, non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        #input
    };
    output.into()
}

#[proc_macro]
pub fn xh(ast: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens : proc_macro2::TokenStream = ast.into();
    quote! { xxhash_rust::xxh3::xxh3_64((#tokens).as_bytes()) }.into()
}

#[proc_macro]
pub fn cxh(ast: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let data = parse_macro_input!(ast as syn::LitStr);
    let item = xxhash_rust::xxh3::xxh3_64(data.value().as_bytes());

    quote! { #item }.into()
}

#[proc_macro_attribute]
pub fn c_enum(
    _metadata: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let output = quote! {
        #[allow(dead_code, non_camel_case_types)]
        #[repr(u8)]
        #[derive(Debug,Clone, Copy, PartialEq, Eq)]
        #input
    };
    output.into()
}

fn to_snake_case_token(input: &str) -> TokenStream {
    cparser_lib::to_snake_case(&input.to_string())
        .parse()
        .expect("Expected snake_base_class correct")
}

#[proc_macro]
pub fn include_c_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    let name = &input.value();
    let mut file = std::fs::File::open(name).expect(&format!("Expected file {} to exist", name));
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)
        .expect(&format!("Expected UTF8 text in {}.xml", name));

    let structs = cparser_lib::parse_cfile(&file_content);
    let mut token_stream = quote::__private::TokenStream::new();
    let mut fields_stream_cache: HashMap<String, quote::__private::TokenStream> = HashMap::new();
    let mut classes_chain: HashMap<String, String> = HashMap::new();

    fields_stream_cache.insert(
        "UObject".to_owned(),
        quote!(
            pub fn uobject(&self) -> ReadResult<UObject> {
                read::<UObject>(self.0)
            }
        ),
    );

    // first pass parses the chain and the fields
    for (_, cstruct) in &structs {
        let mut fields_stream = quote::__private::TokenStream::new();
        let mut field_offset = "";
        let mut field_offset_bitoffset = 0;

        for field in &cstruct.fields {
            let mut mask = if let Some(value) = field.bit_size {
                (1 << (1 + value)) - 1
            } else {
                0
            };

            let shift = if field_offset == field.offset {
                let value = field_offset_bitoffset;
                field_offset_bitoffset += field.bit_size.unwrap_or(0);
                value
            } else {
                field_offset_bitoffset = field.bit_size.unwrap_or(0);
                field_offset = &field.offset;
                0
            };

            mask <<= shift;
            
            let Some(mut ctype) = cparser_lib::get_rust_name(&field.ctype) else {
                continue;
            };

            // this fixes a bug
            if field.bit_size == Some(1) && ctype == "bool" {
                ctype = "u8".to_owned();
            }

            // parsing failed
            if field.name.contains(":") || field.name.contains("[") {
                continue;
            }

            let function_name: TokenStream = to_snake_case_token(&field.name);
            let function_name_write: TokenStream = ("write_".to_owned()
                + &cparser_lib::to_snake_case(&field.name))
                .parse()
                .unwrap();
            let return_type: TokenStream = ctype.parse().unwrap();
            let c_offset: TokenStream = field.offset.parse().unwrap();

            // assuming you have some sort of offset encryption with the custom macro "lu" you can enable it here
            let offset = if cfg!(feature = "encrypt") {
                quote! { lu!(#c_offset) }
            }
            else {
                c_offset
            };

            match field.bit_size {
                Some(1) => {
                    let comment: TokenStream = format!("\"{} & 1<<{shift}\"", field.offset).parse().unwrap();

                    quote!(
                        #[doc=#comment]
                        pub fn #function_name(&self) -> ReadResult<bool> {
                           Ok( (read::<#return_type>(self.0 + #offset)? & #mask as #return_type) != 0)
                        }
        
                        #[doc=#comment]
                        pub fn #function_name_write(&self, value : bool) -> bool {
                            let Ok(read_value) = read::<#return_type>(self.0 + #offset) else {
                                return false;
                            };
                            writef::<#return_type>(self.0 + #offset, 
                                read_value | if value { #mask as #return_type } else { 0 }).is_ok()
                        }
                    )
                    .to_tokens(&mut fields_stream)
                },
                Some(size) => {
                    assert!(size > 0 && size <= 8, "Size should be in the range (0,8]");
                    let comment: TokenStream = format!("\"{} & {:#b} ({}-{})\"", field.offset, mask, shift, size+shift).parse().unwrap();
                    quote!(
                        #[doc=#comment]
                        pub fn #function_name(&self) -> ReadResult<#return_type> {
                            read::<#return_type>(self.0 + #offset)
                        }
        
                        #[doc=#comment]
                        pub fn #function_name_write(&self, value : #return_type) -> bool {
                            let Ok(read_value) = read::<#return_type>(self.0 + #offset) else {
                                return false;
                            };
                            writef::<#return_type>(self.0 + #offset, read_value | (value << #shift)).is_ok()
                        }
                    )
                    .to_tokens(&mut fields_stream)
                }
                None => {
                    let comment: TokenStream = format!("\"{}\"", field.offset).parse().unwrap();
                    quote!(
                        #[doc=#comment]
                        pub fn #function_name(&self) -> ReadResult<#return_type> {
                            read::<#return_type>(self.0 + #offset)
                        }
        
                        #[doc=#comment]
                        pub fn #function_name_write(&self, value : #return_type) -> bool {
                            writef::<#return_type>(self.0 + #offset, value).is_ok()
                        }
                    )
                    .to_tokens(&mut fields_stream)
                },
            }
        }
        if let Some(inherit) = &cstruct.inherit {
            classes_chain.insert(cstruct.name.clone(), inherit.clone());
        }
        fields_stream_cache.insert(cstruct.name.clone(), fields_stream);
    }

    for (_, cstruct) in &structs {
        let mut fields_stream = quote::__private::TokenStream::new();
        let mut inherit = Some(cstruct.name.clone());
        let mut is_uobject = false;
        while let Some(path) = &inherit {
            if path == "UObject" {
                is_uobject = true;
            }
            if let Some(append) = fields_stream_cache.get(path) {
                append.to_tokens(&mut fields_stream);
            }
            inherit = classes_chain.get(path).cloned();
        }

        let name: TokenStream = (cstruct.name.clone() + "Ptr").parse().unwrap();

        let hash_name = xxhash_rust::xxh3::xxh3_64(cstruct.name[1..].as_bytes());
        
        let extra = if is_uobject {
            quote!(
                impl #name {
                    pub fn try_cast<T : Copy + TryCast>(&self) -> ReadResult<T> {
                        if self.uobject()?.is_a_hash(T::HASH) {
                            Ok(self.cast::<T>())
                        } else {
                            Err(MemoryError::BadData)
                        }
                    }

                    pub fn is_a<T : Copy + TryCast>(&self) -> ReadResult<bool> {
                        Ok(self.uobject()?.is_a_hash(T::HASH))
                    }
                }
                
                impl TryCast for #name {
                    const HASH : u64 = #hash_name;
                }
            )
        } else {
            TokenStream::new()
        };

        quote!(
            #[allow(non_camel_case_types)]
            #[repr(transparent)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct #name(UPtr);

            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{:#x}", self.0)
                }
            }

            impl #name {
                #fields_stream

                pub fn cast<T : Copy>(&self) -> T {
                    debug_assert_eq!(std::mem::size_of::<Self>(), std::mem::size_of::<T>());
                    unsafe {
                        let new_value = self.0;
                        *(&new_value as *const UPtr as *const T)
                    }
                }
            }

            #extra

            impl IsValid for #name {
                fn is_valid(&self) -> bool {
                    self.0.is_valid()
                }
            }
        )
        .to_tokens(&mut token_stream)
    }
    token_stream.into()
}