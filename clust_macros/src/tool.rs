extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::collections::BTreeMap;

use quote::{quote, ToTokens};
use syn::{AttrStyle, Expr, ItemFn, Meta};

#[derive(Debug, Clone)]
struct DocComments {
    description: String,
    parameters: BTreeMap<String, ParameterDocument>,
}

#[derive(Debug, Clone)]
struct ParameterDocument {
    name: String,
    description: String,
}

#[derive(Debug, Clone)]
struct ParameterType {
    name: String,
    _type: String,
}

#[derive(Debug, Clone)]
struct Parameter {
    pub name: String,
    pub _type: String,
    pub description: String,
}

impl ToTokens for Parameter {
    fn to_tokens(
        &self,
        tokens: &mut proc_macro2::TokenStream,
    ) {
        let name = &self.name;
        let _type = &self._type;
        let description = &self.description;

        tokens.extend(quote! {
            clust::messages::ParameterElement {
                parameter: clust::messages::Parameter {
                    name: format!(r#"{}"#, #name),
                    _type: format!(r#"{}"#, #_type),
                    description: format!(r#"{}"#, #description),
                },
            }
        });
    }
}

#[derive(Debug, Clone)]
struct ToolInformation {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
}

pub(crate) fn impl_tool(func: &ItemFn) -> TokenStream {
    let tool_information = get_tool_information(func);
    impl_tool_for_function(func, tool_information).into()
}

fn get_doc_comments(func: &ItemFn) -> Vec<String> {
    func.attrs
        .iter()
        .filter_map(|attr| match attr.style {
            | AttrStyle::Outer => {
                if attr
                    .meta
                    .path()
                    .is_ident("doc")
                {
                    match attr.meta.clone() {
                        | Meta::NameValue(meta) => {
                            if let Expr::Lit(lit) = meta.value {
                                Some(
                                    lit.lit
                                        .to_token_stream()
                                        .to_string()
                                        .replace("r\"", "")
                                        .replace("\"", "")
                                        .trim_start_matches(" ")
                                        .to_string(),
                                )
                            } else {
                                None
                            }
                        },
                        | _ => None,
                    }
                } else {
                    None
                }
            },
            | _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocBlockState {
    Description,
    ParametersHeader,
    Parameters,
    Otherwise,
}

impl DocBlockState {
    fn check_block(
        self,
        doc: &str,
    ) -> Self {
        match self {
            | Self::Description => {
                if doc.starts_with("# Arguments")
                    || doc.starts_with("## Arguments")
                {
                    Self::ParametersHeader
                } else {
                    Self::Description
                }
            },
            | Self::Parameters | Self::ParametersHeader => {
                if doc.starts_with("- ") || doc.starts_with("* ") {
                    Self::Parameters
                } else {
                    Self::Otherwise
                }
            },
            | Self::Otherwise => Self::Otherwise,
        }
    }
}

fn parse_doc_comments(docs: Vec<String>) -> DocComments {
    let mut description = String::new();
    let mut parameters = BTreeMap::new();

    let mut state = DocBlockState::Description;

    for doc in docs {
        state = state.check_block(&doc);

        match state {
            | DocBlockState::Description => {
                // Append description
                description.push_str(&doc);
            },
            | DocBlockState::ParametersHeader => continue,
            | DocBlockState::Parameters => {
                // Parse parameters
                let body = doc
                    .trim_start_matches("- ")
                    .trim_start_matches("* ");
                let (parameter_name, parameter_desc) = body.split_at(
                    body.find(" - ")
                        .expect("Parameter description must be in the format `'<name>' - <description>`"),
                );
                let parameter_name = parameter_name.replace("`", "");
                let parameter_desc = parameter_desc.trim_start_matches(" - ");

                parameters.insert(
                    parameter_name.clone(),
                    ParameterDocument {
                        name: parameter_name,
                        description: parameter_desc.to_string(),
                    },
                );
            },
            | DocBlockState::Otherwise => break,
        }
    }

    DocComments {
        description: format!(r#"{}"#, description),
        parameters,
    }
}

fn get_parameter_types(func: &ItemFn) -> Vec<ParameterType> {
    func.sig.inputs.iter().map(|input| {
        match input {
            | syn::FnArg::Typed(pat) => {
                match pat.pat.as_ref() {
                    | syn::Pat::Ident(ident) => {
                        ParameterType {
                            name: ident.ident.to_string(),
                            _type: pat.ty.to_token_stream().to_string(),
                        }
                    },
                    | _ => panic!("Tool trait requires named fields"),
                }
            },
            | _ => panic!("Tool trait can only be derived for functions with named fields, not for methods."),
        }
    }).collect()
}

fn get_tool_information(func: &ItemFn) -> ToolInformation {
    let doc_comments = get_doc_comments(&func);
    let doc_comments = parse_doc_comments(doc_comments);
    let parameter_types = get_parameter_types(&func);

    let parameters = doc_comments
        .parameters
        .iter()
        .map(|(parameter_name, doc_parameter)| {
            let parameter_type = parameter_types
                .iter()
                .find(|parameter_type| {
                    parameter_type.name == parameter_name.as_str()
                })
                .unwrap();

            Parameter {
                name: parameter_name.clone(),
                _type: parameter_type._type.clone(),
                description: doc_parameter
                    .description
                    .clone(),
            }
        })
        .collect();

    ToolInformation {
        name: func.sig.ident.to_string(),
        description: doc_comments.description,
        parameters,
    }
}

fn quote_description(info: &ToolInformation) -> proc_macro2::TokenStream {
    let name = info.name.clone();
    let description = info.description.clone();
    let parameters = info
        .parameters
        .clone()
        .into_iter();

    quote! {
        fn description(&self) -> clust::messages::ToolDescription {
            clust::messages::ToolDescription {
                tool_name: format!(r#"{}"#, #name),
                description: format!(r#"{}"#, #description),
                parameters: vec![
                    #(
                        #parameters,
                    ),*
                ],
            }
        }
    }
}

fn quote_call(
    func: &ItemFn,
    info: &ToolInformation,
) -> proc_macro2::TokenStream {
    let name = info.name.clone();
    let ident = func.sig.ident.clone();
    let parameters: Vec<proc_macro2::TokenStream> = info
        .parameters
        .iter()
        .map(|parameter| parameter.name.clone())
        .map(|parameter| {
            quote! {
                 function_calls.invoke.parameters.get(#parameter)
                        .ok_or_else(|| clust::messages::ToolCallError::ParameterNotFound(#parameter.to_string()))?
                        .parse()
                        .map_err(|_| clust::messages::ToolCallError::ParameterParseFailed(#parameter.to_string()))?
            }
        })
        .collect();

    quote! {
        fn call(&self, function_calls: clust::messages::FunctionCalls)
        -> std::result::Result<clust::messages::FunctionResults, clust::messages::ToolCallError> {
            if function_calls.invoke.tool_name != #name {
                return Err(clust::messages::ToolCallError::ToolNameMismatch);
            }

            let result = #ident(
                #(
                    #parameters
                ),*
            );

            Ok(clust::messages::FunctionResults::Result(
                clust::messages::FunctionResult {
                    tool_name: #name.to_string(),
                    stdout: format!("{}", result),
                }
            ))
        }
    }
}

fn impl_tool_for_function(
    func: &ItemFn,
    info: ToolInformation,
) -> proc_macro2::TokenStream {
    let description_quote = quote_description(&info);
    let call_quote = quote_call(func, &info);
    // let call_quote = quote! {
    //     fn call(&self, function_calls: clust::messages::FunctionCalls)
    //     -> std::result::Result<clust::messages::FunctionResults, clust::messages::ToolCallError> {
    //         unimplemented!()
    //     }
    // };

    let struct_name = format!("ClustTool_{}", info.name);
    let struct_name_ident = Ident::new(&struct_name, Span::call_site());

    quote! {
        #func

        pub struct #struct_name_ident;

        impl clust::messages::Tool for #struct_name_ident {
            #description_quote
            #call_quote
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_doc_comments() {
        let input = quote! {
            /// A function for testing.
            ///
            /// ## Arguments
            /// - `arg1` - First argument.
            fn test_function(arg1: i32) -> i32 {
                arg1
            }
        };

        let item_func = syn::parse_str::<ItemFn>(&input.to_string()).unwrap();
        let doc_comments = get_doc_comments(&item_func);

        assert_eq!(doc_comments.len(), 4);
        assert_eq!(
            doc_comments[0],
            "A function for testing."
        );
        assert_eq!(doc_comments[1], "");
        assert_eq!(doc_comments[2], "## Arguments");
        assert_eq!(
            doc_comments[3],
            "- `arg1` - First argument."
        );
    }

    #[test]
    fn test_parse_doc_comments() {
        let input = quote! {
            /// A function for testing.
            ///
            /// ## Arguments
            /// - `arg1` - First argument.
            fn test_function(arg1: i32) -> i32 {
                arg1
            }
        };

        let item_func = syn::parse_str::<ItemFn>(&input.to_string()).unwrap();
        let doc_comments = get_doc_comments(&item_func);
        let doc_comments = parse_doc_comments(doc_comments);

        assert_eq!(
            doc_comments.description,
            "A function for testing."
        );
        assert_eq!(doc_comments.parameters.len(), 1);
        assert_eq!(
            doc_comments
                .parameters
                .get("arg1")
                .unwrap()
                .name,
            "arg1"
        );
        assert_eq!(
            doc_comments
                .parameters
                .get("arg1")
                .unwrap()
                .description,
            "First argument."
        );
    }

    #[test]
    fn test_get_tool_information() {
        let input = quote! {
            /// A function for testing.
            ///
            /// ## Arguments
            /// - `arg1` - First argument.
            fn test_function(arg1: i32) -> i32 {
                arg1
            }
        };

        let item_func = syn::parse_str::<ItemFn>(&input.to_string()).unwrap();
        let tool_information = get_tool_information(&item_func);

        assert_eq!(tool_information.name, "test_function");
        assert_eq!(
            tool_information.description,
            "A function for testing."
        );
        assert_eq!(
            tool_information
                .parameters
                .len(),
            1
        );
        assert_eq!(
            tool_information
                .parameters
                .get(0)
                .unwrap()
                .name,
            "arg1"
        );
        assert_eq!(
            tool_information
                .parameters
                .get(0)
                .unwrap()
                ._type,
            "i32"
        );
        assert_eq!(
            tool_information
                .parameters
                .get(0)
                .unwrap()
                .description,
            "First argument."
        );
    }
}
