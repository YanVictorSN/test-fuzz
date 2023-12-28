use proc_macro2::Span;
use syn::{
    parse_quote,
    visit_mut::{visit_type_mut, VisitMut},
    Ident, Path, PathArguments, PathSegment, Type, TypePath,
};

pub fn expand_self(trait_path: &Option<Path>, self_ty: &Type, ty: &Type) -> Type {
    let mut ty = ty.clone();
    let mut visitor = ExpandSelfVisitor {
        trait_path,
        self_ty,
    };
    visitor.visit_type_mut(&mut ty);
    ty
}

struct ExpandSelfVisitor<'a> {
    trait_path: &'a Option<Path>,
    self_ty: &'a Type,
}

impl<'a> VisitMut for ExpandSelfVisitor<'a> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        // smoelius: Rewrite this using if-let-guards once the feature is stable.
        // https://rust-lang.github.io/rfcs/2294-if-let-guard.html
        if let Type::Path(path) = ty {
            if match_type_path(path, &["Self"]) == Some(PathArguments::None) {
                *ty = self.self_ty.clone();
                return;
            } else if path.qself.is_none()
                && path
                    .path
                    .segments
                    .first()
                    .map_or(false, |segment| segment.ident == "Self")
            {
                let segments = path.path.segments.iter().skip(1).collect::<Vec<_>>();
                let self_ty = self.self_ty;
                let trait_path = self
                    .trait_path
                    .as_ref()
                    .expect("`trait_path` should be set");
                *ty = parse_quote! { < #self_ty as #trait_path > :: #(#segments)::* };
                return;
            }
        }
        visit_type_mut(self, ty);
    }
}

pub fn match_type_path(path: &TypePath, other: &[&str]) -> Option<PathArguments> {
    let mut path = path.clone();
    let args = path.path.segments.last_mut().map(|segment| {
        let args = segment.arguments.clone();
        segment.arguments = PathArguments::None;
        args
    });
    let lhs = path.path.segments.into_iter().collect::<Vec<_>>();
    let rhs = other
        .iter()
        .map(|s| {
            let ident = Ident::new(s, Span::call_site());
            PathSegment {
                ident,
                arguments: PathArguments::None,
            }
        })
        .collect::<Vec<_>>();
    if path.qself.is_none() && lhs == rhs {
        args
    } else {
        None
    }
}

pub fn type_base(ty: &Type) -> Type {
    let mut ty = ty.clone();

    if let Type::Path(ref mut path) = ty {
        if let Some(segment) = path.path.segments.last_mut() {
            let ident = &segment.ident;
            *segment = parse_quote! { #ident };
        }
    }

    ty
}
