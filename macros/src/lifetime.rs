use proc_macro2::Span;
use syn::visit_mut::{self, VisitMut};
use syn::{GenericArgument, Lifetime, Receiver, Token, TypeReference};

#[derive(Default)]
pub struct CollectLifetimes {
    pub elided: Vec<Lifetime>,
    pub explicit: Vec<Lifetime>,
}

impl CollectLifetimes {
    fn visit_opt_lifetime(&mut self, reference: Token![&], lifetime: &mut Option<Lifetime>) {
        match lifetime {
            None => *lifetime = Some(self.next_lifetime(reference.span)),
            Some(lifetime) => self.visit_lifetime(lifetime),
        }
    }

    fn visit_lifetime(&mut self, lifetime: &mut Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = self.next_lifetime(lifetime.span());
        } else {
            self.explicit.push(lifetime.clone());
        }
    }

    fn next_lifetime(&mut self, span: Span) -> Lifetime {
        let name = format!("'life{}", self.elided.len());
        let life = Lifetime::new(&name, span);
        self.elided.push(life.clone());
        life
    }
}

impl VisitMut for CollectLifetimes {
    fn visit_receiver_mut(&mut self, arg: &mut Receiver) {
        if let Some((reference, lifetime)) = &mut arg.reference {
            self.visit_opt_lifetime(*reference, lifetime);
        } else {
            visit_mut::visit_type_mut(self, &mut arg.ty);
        }
    }

    fn visit_type_reference_mut(&mut self, ty: &mut TypeReference) {
        self.visit_opt_lifetime(ty.and_token, &mut ty.lifetime);
        visit_mut::visit_type_reference_mut(self, ty);
    }

    fn visit_generic_argument_mut(&mut self, gen: &mut GenericArgument) {
        if let GenericArgument::Lifetime(lifetime) = gen {
            self.visit_lifetime(lifetime);
        }
        visit_mut::visit_generic_argument_mut(self, gen);
    }
}
