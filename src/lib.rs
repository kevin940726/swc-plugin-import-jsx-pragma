use serde::Deserialize;
use swc_core::common::{Mark, Span, SyntaxContext, DUMMY_SP};
use swc_core::ecma::{
    ast::*,
    atoms::JsWord,
    utils::quote_ident,
    visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct TransformVisitor {
    has_jsx_element: bool,
    has_jsx_fragment: bool,
    import_source: JsWord,
}

impl Default for TransformVisitor {
    fn default() -> Self {
        Self {
            has_jsx_element: false,
            has_jsx_fragment: false,
            import_source: String::from("react").into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginConfig {
    import_source: Option<String>,
}

impl VisitMut for TransformVisitor {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
    fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
        self.has_jsx_element = true;
        n.visit_mut_children_with(self);
    }
    fn visit_mut_jsx_fragment(&mut self, n: &mut JSXFragment) {
        self.has_jsx_fragment = true;
        n.visit_mut_children_with(self);
    }
    fn visit_mut_module_items(&mut self, n: &mut Vec<ModuleItem>) {
        n.visit_mut_children_with(self);

        if self.has_jsx_element || self.has_jsx_fragment {
            let mut specifiers: Vec<ImportSpecifier> = vec![];
            let span = Span {
                ctxt: SyntaxContext::empty().apply_mark(Mark::from_u32(2)),
                ..DUMMY_SP
            };

            if self.has_jsx_element {
                specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: quote_ident!(span, "createElement"),
                    imported: None,
                    is_type_only: false,
                }));
            }
            if self.has_jsx_fragment {
                specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: quote_ident!(span, "Fragment"),
                    imported: None,
                    is_type_only: false,
                }));
            }

            n.insert(
                0,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers,
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: self.import_source.clone(),
                        raw: None,
                    }),
                    type_only: false,
                    asserts: None,
                })),
            );
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<PluginConfig>(
        &metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config");

    let import_source = config.import_source.unwrap_or(String::from("react")).into();

    program.fold_with(&mut as_folder(TransformVisitor {
        has_jsx_element: false,
        has_jsx_fragment: false,
        import_source,
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::ecma::{
        parser::{EsConfig, Syntax},
        transforms::testing::test,
    };

    fn syntax() -> Syntax {
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        })
    }

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            ..Default::default()
        }),
        test_no_transform,
        // Input codes
        r#"console.log('No transform')"#,
        // Output codes after transformed with plugin
        r#"console.log('No transform')"#
    );

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            ..Default::default()
        }),
        test_element_pragma,
        // Input codes
        r#"const jsx = <div />"#,
        // Output codes after transformed with plugin
        r#"import { createElement } from "react";
const jsx = <div />"#
    );

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            ..Default::default()
        }),
        test_component_pragma,
        // Input codes
        r#"const jsx = <Component />"#,
        // Output codes after transformed with plugin
        r#"import { createElement } from "react";
const jsx = <Component />"#
    );

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            ..Default::default()
        }),
        test_fragment,
        // Input codes
        r#"const jsx = <></>"#,
        // Output codes after transformed with plugin
        r#"import { Fragment } from "react";
const jsx = <></>"#
    );

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            ..Default::default()
        }),
        test_pragma_and_fragment,
        // Input codes
        r#"const jsx = <><div /></>"#,
        // Output codes after transformed with plugin
        r#"import { createElement, Fragment } from "react";
const jsx = <><div /></>"#
    );

    test!(
        syntax(),
        |_| as_folder(TransformVisitor {
            import_source: "preact".into(),
            ..Default::default()
        }),
        test_import_source,
        // Input codes
        r#"const jsx = <div />"#,
        // Output codes after transformed with plugin
        r#"import { createElement } from "preact";
const jsx = <div />"#
    );
}
