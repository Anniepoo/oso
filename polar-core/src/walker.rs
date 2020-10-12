use super::partial::Constraints;
use super::terms::*;

pub trait Visitor<'term>: Sized {
    // Atoms. These may be overridden as needed.
    fn visit_number(&mut self, _n: &'term Numeric) {}
    fn visit_string(&mut self, _s: &'term str) {}
    fn visit_boolean(&mut self, _b: bool) {}
    fn visit_id(&mut self, _id: u64) {}
    fn visit_name(&mut self, _tag: &'term Symbol) {}
    fn visit_field(&mut self, _name: &'term Symbol, _value: &'term Term) {}
    fn visit_variable(&mut self, _v: &'term Symbol) {}
    fn visit_rest_variable(&mut self, _r: &'term Symbol) {}
    fn visit_operator(&mut self, _o: &'term Operator) {}

    // Compounds. If you override these, you must walk the children manually.
    fn visit_term(&mut self, t: &'term Term) {
        walk_term(self, t)
    }
    fn visit_external_instance(&mut self, e: &'term ExternalInstance) {
        walk_external_instance(self, e)
    }
    fn visit_instance_literal(&mut self, i: &'term InstanceLiteral) {
        walk_instance_literal(self, i)
    }
    fn visit_dictionary(&mut self, d: &'term Dictionary) {
        walk_dictionary(self, d)
    }
    fn visit_pattern(&mut self, p: &'term Pattern) {
        walk_pattern(self, p)
    }
    fn visit_call(&mut self, c: &'term Call) {
        walk_call(self, c)
    }
    #[allow(clippy::ptr_arg)]
    fn visit_list(&mut self, l: &'term TermList) {
        walk_list(self, l)
    }
    fn visit_operation(&mut self, o: &'term Operation) {
        walk_operation(self, o)
    }
    fn visit_partial(&mut self, c: &'term Constraints) {
        walk_partial(self, c)
    }
}

pub fn walk_term<'a, V: Visitor<'a>>(visitor: &mut V, term: &'a Term) {
    match term.value() {
        Value::Number(n) => visitor.visit_number(n),
        Value::String(s) => visitor.visit_string(s),
        Value::Boolean(b) => visitor.visit_boolean(*b),
        Value::ExternalInstance(e) => visitor.visit_external_instance(e),
        Value::InstanceLiteral(i) => visitor.visit_instance_literal(i),
        Value::Dictionary(d) => visitor.visit_dictionary(d),
        Value::Pattern(p) => visitor.visit_pattern(p),
        Value::Call(c) => visitor.visit_call(c),
        Value::List(l) => visitor.visit_list(l),
        Value::Variable(v) => visitor.visit_variable(v),
        Value::RestVariable(r) => visitor.visit_rest_variable(r),
        Value::Expression(o) => visitor.visit_operation(o),
        Value::Partial(p) => visitor.visit_partial(p),
    }
}

pub fn walk_external_instance<'a, V: Visitor<'a>>(visitor: &mut V, instance: &'a ExternalInstance) {
    visitor.visit_id(instance.instance_id);
}

macro_rules! walk_elements {
    ($visitor: expr, $method: ident, $list: expr) => {
        for element in $list {
            $visitor.$method(element)
        }
    };
}

macro_rules! walk_fields {
    ($visitor: expr, $method: ident, $dict: expr) => {
        for (k, v) in $dict {
            $visitor.$method(k, v)
        }
    };
}

pub fn walk_instance_literal<'a, V: Visitor<'a>>(visitor: &mut V, instance: &'a InstanceLiteral) {
    visitor.visit_name(&instance.tag);
    walk_fields!(visitor, visit_field, &instance.fields.fields);
}

pub fn walk_dictionary<'a, V: Visitor<'a>>(visitor: &mut V, dict: &'a Dictionary) {
    walk_fields!(visitor, visit_field, &dict.fields);
}

pub fn walk_pattern<'a, V: Visitor<'a>>(visitor: &mut V, pattern: &'a Pattern) {
    match pattern {
        Pattern::Dictionary(dict) => walk_fields!(visitor, visit_field, &dict.fields),
        Pattern::Instance(instance) => visitor.visit_instance_literal(&instance),
    }
}

pub fn walk_call<'a, V: Visitor<'a>>(visitor: &mut V, call: &'a Call) {
    visitor.visit_name(&call.name);
    walk_elements!(visitor, visit_term, &call.args);
}

#[allow(clippy::ptr_arg)]
pub fn walk_list<'a, V: Visitor<'a>>(visitor: &mut V, list: &'a TermList) {
    walk_elements!(visitor, visit_term, list);
}

pub fn walk_operation<'a, V: Visitor<'a>>(visitor: &mut V, expr: &'a Operation) {
    visitor.visit_operator(&expr.operator);
    walk_elements!(visitor, visit_term, &expr.args);
}

pub fn walk_partial<'a, V: Visitor<'a>>(visitor: &mut V, partial: &'a Constraints) {
    visitor.visit_name(&partial.variable);
    walk_elements!(visitor, visit_operation, &partial.operations);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestVisitor {
        visited: Vec<Value>,
    }

    impl TestVisitor {
        fn new() -> Self {
            Self { visited: vec![] }
        }
        fn push(&mut self, value: Value) {
            self.visited.push(value);
        }
    }

    impl<'term> Visitor<'term> for TestVisitor {
        fn visit_number(&mut self, n: &'term Numeric) {
            self.push(Value::Number(*n));
        }
        fn visit_string(&mut self, s: &'term str) {
            self.push(Value::String(s.to_string()));
        }
        fn visit_boolean(&mut self, b: bool) {
            self.push(Value::Boolean(b));
        }
    }

    #[test]
    fn test_walk_term() {
        let t = term!([value!(1), value!("Hi there!"), value!(true)]);
        let mut v = TestVisitor::new();
        v.visit_term(&t);
        assert_eq!(
            v.visited,
            vec![value!(1), value!("Hi there!"), value!(true)]
        );
    }
}