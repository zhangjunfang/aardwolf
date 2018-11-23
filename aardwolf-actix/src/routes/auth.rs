use aardwolf_models::user::UserLike;
use aardwolf_templates::templates;
use aardwolf_types::forms::auth::{
    ConfirmAccountFail, ConfirmToken, ConfirmationToken, SignIn, SignInErrorMessage, SignInFail,
    SignInForm, SignUp, SignUpFail, SignUpForm, ValidateSignInForm, ValidateSignInFormFail,
    ValidateSignUpForm, ValidateSignUpFormFail,
};
use actix_web::{
    http::header::LOCATION, middleware::session::Session, Form, HttpResponse, Query, State,
};
use failure::Fail;
use futures::future::Future;
use rocket_i18n::I18n;

use crate::{db::DbActionError, error::RedirectError, types::user::SignedInUser, AppConfig};

pub(crate) fn sign_up_form((state, i18n): (State<AppConfig>, I18n)) -> HttpResponse {
    state.render(move |buf| {
        templates::sign_up(
            buf,
            &i18n.catalog,
            Default::default(),
            "csrf token",
            None,
            None,
        )
    })
}

pub(crate) fn sign_in_form(
    (state, error, i18n): (State<AppConfig>, Option<Query<SignInErrorMessage>>, I18n),
) -> HttpResponse {
    state.render(move |buf| {
        templates::sign_in(
            buf,
            &i18n.catalog,
            "csrf token",
            error.map(|e| e.into_inner()),
        )
    })
}

#[derive(Clone, Debug, Fail)]
pub enum SignUpError {
    #[fail(display = "Error talking to db actor")]
    Mailbox,
    #[fail(display = "Error talking db")]
    Database,
    #[fail(display = "Error signing up: {}", _0)]
    SignUp(#[cause] SignUpFail),
}

impl From<DbActionError<SignUpFail>> for SignUpError {
    fn from(e: DbActionError<SignUpFail>) -> Self {
        match e {
            DbActionError::Connection => SignUpError::Database,
            DbActionError::Mailbox => SignUpError::Mailbox,
            DbActionError::Action(e) => SignUpError::SignUp(e),
        }
    }
}

impl From<ValidateSignUpFormFail> for SignUpError {
    fn from(e: ValidateSignUpFormFail) -> Self {
        SignUpError::SignUp(e.into())
    }
}

pub(crate) fn sign_up(
    (state, form, i18n): (State<AppConfig>, Form<SignUpForm>, I18n),
) -> Box<dyn Future<Item = HttpResponse, Error = actix_web::Error>> {
    let state2 = state.clone();
    let form = form.into_inner();
    let form_state = form.as_state();

    let res = perform!(state, SignUpError, [
        (form = ValidateSignUpForm(form)),
        (_ = SignUp(form)),
    ]);

    Box::new(
        res.map(|(email, token)| {
            println!(
                "confirmation token url: /auth/confirmation?id={}&token={}",
                email.id(),
                token
            );

            HttpResponse::SeeOther()
                .header(LOCATION, "/auth/sign_in")
                .finish()
        })
        .or_else(move |e| match e {
            SignUpError::SignUp(e) => match e {
                SignUpFail::ValidationError(e) => Ok(state2.render(move |buf| {
                    templates::sign_up(buf, &i18n.catalog, form_state, "csrf token", Some(e), None)
                })),
                e => Ok(state2.render(move |buf| {
                    templates::sign_up(
                        buf,
                        &i18n.catalog,
                        form_state,
                        "csrf token",
                        None,
                        Some(format!("{}", e)),
                    )
                })),
            },
            e => Ok(state2.render(move |buf| {
                templates::sign_up(
                    buf,
                    &i18n.catalog,
                    form_state,
                    "csrf token",
                    None,
                    Some(format!("{}", e)),
                )
            })),
        }),
    )
}

#[derive(Clone, Debug, Fail)]
pub enum SignInError {
    #[fail(display = "Error talking to db actor")]
    Mailbox,
    #[fail(display = "Error talking db")]
    Database,
    #[fail(display = "Error signing in: {}", _0)]
    SignIn(#[cause] SignInFail),
}

impl From<DbActionError<SignInFail>> for SignInError {
    fn from(e: DbActionError<SignInFail>) -> Self {
        match e {
            DbActionError::Connection => SignInError::Database,
            DbActionError::Mailbox => SignInError::Mailbox,
            DbActionError::Action(e) => SignInError::SignIn(e),
        }
    }
}

impl From<ValidateSignInFormFail> for SignInError {
    fn from(e: ValidateSignInFormFail) -> Self {
        SignInError::SignIn(e.into())
    }
}

pub(crate) fn sign_in(
    (state, session, form): (State<AppConfig>, Session, Form<SignInForm>),
) -> Box<dyn Future<Item = HttpResponse, Error = actix_web::Error>> {
    let res = perform!(state, SignInError, [
        (form = ValidateSignInForm(form.into_inner())),
        (_ = SignIn(form)),
    ]);

    Box::new(
        res.map_err(|e| RedirectError::new("/auth/sign_in", &Some(e.to_string())).into())
            .and_then(move |user| {
                session
                    .set("user_id", user.id())
                    .map_err(|e| RedirectError::new("/auth/sign_in", &Some(e.to_string())).into())
            })
            .map(|_| HttpResponse::SeeOther().header(LOCATION, "/").finish()),
    )
}

#[derive(Clone, Debug, Fail)]
pub enum ConfirmError {
    #[fail(display = "Error talking to db actor")]
    Mailbox,
    #[fail(display = "Error talking db")]
    Database,
    #[fail(display = "Error confirming account: {}", _0)]
    Confirm(#[cause] ConfirmAccountFail),
}

impl From<DbActionError<ConfirmAccountFail>> for ConfirmError {
    fn from(e: DbActionError<ConfirmAccountFail>) -> Self {
        match e {
            DbActionError::Connection => ConfirmError::Database,
            DbActionError::Mailbox => ConfirmError::Mailbox,
            DbActionError::Action(e) => ConfirmError::Confirm(e),
        }
    }
}

pub(crate) fn confirm(
    (state, query): (State<AppConfig>, Query<ConfirmationToken>),
) -> Box<dyn Future<Item = HttpResponse, Error = actix_web::Error>> {
    let res = perform!(state, ConfirmError, [
        (_ = ConfirmToken(query.into_inner())),
    ]);

    Box::new(
        res.map(|_user| {
            HttpResponse::SeeOther()
                .header(LOCATION, "/auth/sign_in")
                .finish()
        })
        .map_err(|e| RedirectError::new("/auth/sign_up", &Some(e.to_string())).into()),
    )
}

pub(crate) fn sign_out((session, _user): (Session, SignedInUser)) -> HttpResponse {
    session.remove("user_id");

    HttpResponse::SeeOther()
        .header(LOCATION, "/auth/sign_in")
        .finish()
}
