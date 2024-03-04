use crate::generate_route_list::RouteList;
use core::marker::PhantomData;
use either_of::Either;
use routing_utils::{
    location::Location,
    matching::{MatchNestedRoutes, PossibleRouteMatch, Routes},
};
use std::borrow::Cow;
use tachys::{
    hydration::Cursor,
    renderer::Renderer,
    ssr::StreamBuilder,
    view::{either::EitherState, Position, PositionState, Render, RenderHtml},
};

#[derive(Debug)]
pub struct Router<Rndr, Loc, Children, FallbackFn> {
    base: Option<Cow<'static, str>>,
    location: Loc,
    routes: Routes<Children>,
    fallback: FallbackFn,
    rndr: PhantomData<Rndr>,
}

impl<Rndr, Loc, Children, FallbackFn, Fallback>
    Router<Rndr, Loc, Children, FallbackFn>
where
    Loc: Location,
    Rndr: Renderer,
    FallbackFn: Fn() -> Fallback,
{
    pub fn new(
        location: Loc,
        routes: Routes<Children>,
        fallback: FallbackFn,
    ) -> Router<Rndr, Loc, Children, FallbackFn> {
        Self {
            base: None,
            location,
            routes,
            fallback,
            rndr: PhantomData,
        }
    }

    pub fn new_with_base(
        base: impl Into<Cow<'static, str>>,
        location: Loc,
        routes: Routes<Children>,
        fallback: FallbackFn,
    ) -> Router<Rndr, Loc, Children, FallbackFn> {
        Self {
            base: Some(base.into()),
            location,
            routes,
            fallback,
            rndr: PhantomData,
        }
    }

    pub fn set_location(&mut self, new_location: Loc) {
        self.location = new_location;
    }
}

impl<Rndr, Loc, Children, FallbackFn, Fallback>
    Router<Rndr, Loc, Children, FallbackFn>
where
    FallbackFn: Fn() -> Fallback,
    Rndr: Renderer,
{
    pub fn fallback(&self) -> Fallback {
        (self.fallback)()
    }
}

impl<Rndr, Loc, FallbackFn, Fallback, Children> Render<Rndr>
    for Router<Rndr, Loc, Children, FallbackFn>
where
    Loc: Location,
    FallbackFn: Fn() -> Fallback,
    Fallback: Render<Rndr>,
    for<'a> Children: MatchNestedRoutes<'a>,
    for<'a> <Children as MatchNestedRoutes<'a>>::Match: core::fmt::Debug,
    Rndr: Renderer,
{
    type State = EitherState<
        <String as Render<Rndr>>::State,
        <Fallback as Render<Rndr>>::State,
        Rndr,
    >;
    type FallibleState = ();

    fn build(self) -> Self::State {
        match self
            .location
            .current()
            .as_ref()
            .map(|url| self.routes.match_route(url.path()))
        {
            Ok(Some(matched)) => Either::Left(format!("{matched:#?}")),
            _ => Either::Right((self.fallback)()),
        }
        .build()
    }

    fn rebuild(self, state: &mut Self::State) {
        todo!()
    }

    fn try_build(self) -> tachys::error::Result<Self::FallibleState> {
        todo!()
    }

    fn try_rebuild(
        self,
        state: &mut Self::FallibleState,
    ) -> tachys::error::Result<()> {
        todo!()
    }
}
/*
impl<Rndr, Loc, Fal, Children> RenderHtml<Rndr>
    for Router<Rndr, Loc, Children, Fal>
where
    Self: FallbackOrViewHtml,
    Rndr: Renderer,
    Loc: Location,
    Children: PossibleRouteMatch,
    <Self as FallbackOrView>::Output: RenderHtml<Rndr>,
    Rndr::Element: Clone,
    Rndr::Node: Clone,
{
    const MIN_LENGTH: usize = <Self as FallbackOrViewHtml>::MIN_LENGTH;

    fn to_html_with_buf(self, buf: &mut String, position: &mut Position) {
        if RouteList::is_generating() {
            let routes = RouteList::default();
            RouteList::register(routes);
        } else {
            self.fallback_or_view().1.to_html_with_buf(buf, position);
        }
    }

    fn to_html_async_with_buf<const OUT_OF_ORDER: bool>(
        self,
        buf: &mut StreamBuilder,
        position: &mut Position,
    ) where
        Self: Sized,
    {
        if RouteList::is_generating() {
            let routes = RouteList::default();
            RouteList::register(routes);
        } else {
            self.fallback_or_view()
                .1
                .to_html_async_with_buf::<OUT_OF_ORDER>(buf, position);
        }
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        cursor: &Cursor<Rndr>,
        position: &PositionState,
    ) -> Self::State {
        self.fallback_or_view()
            .1
            .hydrate::<FROM_SERVER>(cursor, position)
    }
}

pub trait FallbackOrView {
    type Output;

    fn fallback_or_view(&self) -> (&'static str, Self::Output);

    fn generate_route_list(&self, route_list: &mut RouteList);
}

pub trait FallbackOrViewHtml: FallbackOrView {
    const MIN_LENGTH: usize;
}

impl<Rndr, Loc, FallbackFn, Fal> FallbackOrView
    for Router<Rndr, Loc, (), FallbackFn>
where
    Rndr: Renderer,
    Loc: Location,
    FallbackFn: Fn() -> Fal,
    Fal: Render<Rndr>,
{
    type Output = Fal;

    fn fallback_or_view(&self) -> (&'static str, Self::Output) {
        ("Fal", (self.fallback)())
    }

    fn generate_route_list(&self, _route_list: &mut RouteList) {}
}

impl<Rndr, Loc, FallbackFn, Fal> FallbackOrViewHtml
    for Router<Rndr, Loc, (), FallbackFn>
where
    Rndr: Renderer,
    Loc: Location,
    FallbackFn: Fn() -> Fal,
    Fal: RenderHtml<Rndr>,
    Rndr::Element: Clone,
    Rndr::Node: Clone,
{
    const MIN_LENGTH: usize = Fal::MIN_LENGTH;
}

impl<Rndr, Loc, FallbackFn, Fal, APat, AViewFn, AView, AChildren> FallbackOrView
    for Router<
        Rndr,
        Loc,
        RouteDefinition<Rndr, APat, AViewFn, AChildren>,
        FallbackFn,
    >
where
    Rndr: Renderer,
    Loc: Location,
    APat: RouteMatch,
    AViewFn: Fn(MatchedRoute) -> AView,
    AView: Render<Rndr>,
    FallbackFn: Fn() -> Fal,
    Fal: Render<Rndr>,
{
    type Output = Either<Fal, AView>;

    fn fallback_or_view(&self) -> (&'static str, Self::Output) {
        match self.location.try_to_url() {
            Ok(url) => {
                if self.routes.path.matches(&url.pathname) {
                    let PartialPathMatch {
                        params,
                        matched,
                        remaining,
                    } = self.routes.path.test(&url.pathname).unwrap();
                    if remaining.is_empty() {
                        let matched = MatchedRoute {
                            params,
                            matched,
                            search_params: url.search_params.clone(),
                        };
                        return (
                            "Route",
                            Either::Right(self.routes.view(matched)),
                        );
                    }
                }
                ("Fal", Either::Left(self.fallback()))
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                {
                    tracing::error!(
                        "Error converting location into URL: {e:?}"
                    );
                }
                ("Fal", Either::Left(self.fallback()))
            }
        }
    }

    fn generate_route_list(&self, route_list: &mut RouteList) {
        let mut path = Vec::new();
        self.routes.path.generate_path(&mut path);
        route_list.push(RouteListing::from_path(path));
    }
}

impl<Rndr, Loc, FallbackFn, Fal, APat, AViewFn, AView, AChildren>
    FallbackOrViewHtml
    for Router<
        Rndr,
        Loc,
        RouteDefinition<Rndr, APat, AViewFn, AChildren>,
        FallbackFn,
    >
where
    Rndr: Renderer,
    Loc: Location,
    APat: RouteMatch,
    AViewFn: Fn(MatchedRoute) -> AView,
    AView: RenderHtml<Rndr>,
    FallbackFn: Fn() -> Fal,
    Fal: RenderHtml<Rndr>,
    Rndr::Element: Clone,
    Rndr::Node: Clone,
{
    const MIN_LENGTH: usize = if Fal::MIN_LENGTH < AView::MIN_LENGTH {
        Fal::MIN_LENGTH
    } else {
        AView::MIN_LENGTH
    };
}*/
