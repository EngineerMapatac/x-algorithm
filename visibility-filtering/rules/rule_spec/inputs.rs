use crate::hydration::Hydrators;
use crate::rules::RuleContext;

pub(in crate::rules) trait Inputs<'a>: Sized {
    const REQUIRED: Hydrators;
    fn acquire(context: &RuleContext<'a>) -> Self;
}

pub(in crate::rules) trait Interpreter<'a> {
    type Output<T>;
    fn run<A: Inputs<'a>, T>(&self, kernel: impl FnOnce(A) -> T) -> Self::Output<T>;
}

pub(in crate::rules) struct Required;

impl<'a> Interpreter<'a> for Required {
    type Output<T> = Hydrators;

    fn run<A: Inputs<'a>, T>(&self, _: impl FnOnce(A) -> T) -> Hydrators {
        A::REQUIRED
    }
}

pub(in crate::rules) struct EvaluateInputs<'c, 'a>(pub(in crate::rules) &'c RuleContext<'a>);

pub(in crate::rules) struct Acquired<T> {
    pub(in crate::rules) value: T,
    pub(in crate::rules) failed: Hydrators,
}

impl<'a> Interpreter<'a> for EvaluateInputs<'_, 'a> {
    type Output<T> = Acquired<T>;

    #[inline]
    fn run<A: Inputs<'a>, T>(&self, kernel: impl FnOnce(A) -> T) -> Acquired<T> {
        let inputs = A::acquire(self.0);
        Acquired {
            value: kernel(inputs),
            failed: self.0.failed().intersection(A::REQUIRED),
        }
    }
}

impl<'a> Inputs<'a> for () {
    const REQUIRED: Hydrators = Hydrators::empty();
    fn acquire(_: &RuleContext<'a>) -> Self {}
}

impl<'a, A: Inputs<'a>, B: Inputs<'a>> Inputs<'a> for (A, B) {
    const REQUIRED: Hydrators = A::REQUIRED.union(B::REQUIRED);
    #[inline]
    fn acquire(context: &RuleContext<'a>) -> Self {
        (A::acquire(context), B::acquire(context))
    }
}

impl<'a, A: Inputs<'a>, B: Inputs<'a>, C: Inputs<'a>> Inputs<'a> for (A, B, C) {
    const REQUIRED: Hydrators = A::REQUIRED.union(B::REQUIRED).union(C::REQUIRED);
    #[inline]
    fn acquire(context: &RuleContext<'a>) -> Self {
        (
            A::acquire(context),
            B::acquire(context),
            C::acquire(context),
        )
    }
}

impl<'a, A: Inputs<'a>, B: Inputs<'a>, C: Inputs<'a>, D: Inputs<'a>> Inputs<'a> for (A, B, C, D) {
    const REQUIRED: Hydrators = A::REQUIRED
        .union(B::REQUIRED)
        .union(C::REQUIRED)
        .union(D::REQUIRED);
    #[inline]
    fn acquire(context: &RuleContext<'a>) -> Self {
        (
            A::acquire(context),
            B::acquire(context),
            C::acquire(context),
            D::acquire(context),
        )
    }
}
