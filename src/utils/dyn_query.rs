use diesel::backend::Backend;
use diesel::dsl::{Asc, Desc};
use diesel::helper_types::IntoBoxed;
use diesel::query_dsl::methods::{BoxedDsl, OrderDsl};
use diesel::{ExpressionMethods, QueryDsl};

#[derive(Clone, Copy)]
pub enum QueryOrdering {
    Ascending,
    Descending,
}

impl QueryOrdering {
    fn order_query<'a, Expr, Q, DB>(&self, query: Q, expr: Expr) -> IntoBoxed<'a, Q, DB>
        where
            Expr: ExpressionMethods,
            Q: QueryDsl + BoxedDsl<'a, DB>,
            DB: Backend,
            IntoBoxed<'a, Q, DB>: OrderDsl<Asc<Expr>, Output = IntoBoxed<'a, Q, DB>>
            + OrderDsl<Desc<Expr>, Output = IntoBoxed<'a, Q, DB>>,
    {
        match self {
            Self::Ascending => query.into_boxed().order(expr.asc()),
            Self::Descending => query.into_boxed().order(expr.desc()),
        }
    }
    fn order_query2<'a, Expr, Q, DB>(
        &self,
        query: Q,
        expr: Expr,
    ) -> <Q as OrderDsl<Asc<Expr>>>::Output
        where
            Expr: ExpressionMethods,
            DB: Backend,
            Q: OrderDsl<Asc<Expr>>
            + OrderDsl<Desc<Expr>, Output = <Q as OrderDsl<Asc<Expr>>>::Output>,
    {
        match self {
            Self::Ascending => query.order(expr.asc()),
            Self::Descending => query.order(expr.desc()),
        }
    }
}

pub trait DynOrderDsl2<'a, DB, Expr>
    where
        Expr: ExpressionMethods,
        DB: Backend,
        Self: QueryDsl
        + OrderDsl<Asc<Expr>>
        + OrderDsl<Desc<Expr>, Output = <Self as OrderDsl<Asc<Expr>>>::Output>,
{
    fn dyn_order2(
        self,
        order: QueryOrdering,
        expr: Expr,
    ) -> <Self as OrderDsl<Asc<Expr>>>::Output;
}

impl<'a, Expr, DB, Q> DynOrderDsl2<'a, DB, Expr> for Q
    where
        Expr: ExpressionMethods,
        DB: Backend,
        Q: QueryDsl
        + OrderDsl<Asc<Expr>>
        + OrderDsl<Desc<Expr>, Output = <Q as OrderDsl<Asc<Expr>>>::Output>,
{
    fn dyn_order2(
        self,
        order: QueryOrdering,
        expr: Expr,
    ) -> <Q as OrderDsl<Asc<Expr>>>::Output {
        order.order_query2::<_, _, DB>(self, expr)
    }
}

pub trait DynOrderDsl<'a, DB, Expr>
    where
        Expr: ExpressionMethods,
        Self: QueryDsl + BoxedDsl<'a, DB>,
        DB: Backend,
        IntoBoxed<'a, Self, DB>: OrderDsl<Asc<Expr>, Output = IntoBoxed<'a, Self, DB>>
        + OrderDsl<Desc<Expr>, Output = IntoBoxed<'a, Self, DB>>,
{
    fn dyn_order(self, order: QueryOrdering, expr: Expr) -> IntoBoxed<'a, Self, DB>;
}

impl<'a, Expr, DB, Q> DynOrderDsl<'a, DB, Expr> for Q
    where
        Expr: ExpressionMethods,
        Q: QueryDsl + BoxedDsl<'a, DB>,
        DB: Backend,
        IntoBoxed<'a, Q, DB>: OrderDsl<Asc<Expr>, Output = IntoBoxed<'a, Q, DB>>
        + OrderDsl<Desc<Expr>, Output = IntoBoxed<'a, Q, DB>>,
{
    fn dyn_order(self, order: QueryOrdering, expr: Expr) -> IntoBoxed<'a, Q, DB> {
        order.order_query(self, expr)
    }
}

pub trait ExtendedQueryDsl: Sized {
    fn order_dyn<'a, DB, Expr>(self, order: QueryOrdering, expr: Expr) -> IntoBoxed<'a, Self, DB>
        where
            Expr: ExpressionMethods,
            Self: QueryDsl + BoxedDsl<'a, DB>,
            DB: Backend,
            IntoBoxed<'a, Self, DB>: OrderDsl<Asc<Expr>, Output = IntoBoxed<'a, Self, DB>>
            + OrderDsl<Desc<Expr>, Output = IntoBoxed<'a, Self, DB>>,
    {
        DynOrderDsl::<DB, Expr>::dyn_order(self, order, expr)
    }
    fn order_dyn2<DB, Expr>(
        self,
        order: QueryOrdering,
        expr: Expr,
    ) -> <Self as OrderDsl<Asc<Expr>>>::Output
        where
            Expr: ExpressionMethods,
            DB: Backend,
            Self: QueryDsl
            + OrderDsl<Asc<Expr>>
            + OrderDsl<Desc<Expr>, Output = <Self as OrderDsl<Asc<Expr>>>::Output>,
    {
        DynOrderDsl2::<DB, Expr>::dyn_order2(self, order, expr)
    }
}

impl<Q> ExtendedQueryDsl for Q {}