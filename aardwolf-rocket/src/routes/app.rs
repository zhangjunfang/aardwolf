use std::path::{Path, PathBuf};

use aardwolf_models::{sql_types::PostVisibility, user::UserLike};
use aardwolf_types::forms::posts::PostCreationFormState;
use rocket::{
    http::Cookies,
    response::{status::NotFound, NamedFile, Redirect},
};
use rocket_i18n::I18n;

use crate::{render_template, types::user::SignedInUser, DbConn, ResponseOrRedirect};

#[get("/")]
pub fn home(
    user: SignedInUser,
    mut cookies: Cookies,
    i18n: I18n,
    _db: DbConn,
) -> ResponseOrRedirect {
    let res = if cookies.get_private("persona_id").is_some() || user.0.primary_persona().is_some() {
        render_template(&aardwolf_templates::Home::new(
            &i18n.catalog,
            user.0.id().to_string().as_ref(),
            user.0.id().to_string().as_ref(),
            "csrf token",
            &PostCreationFormState {
                source: "".to_owned(),
                name: None,
                visibility: PostVisibility::Public, // TODO: this comes from the persona
            },
            None,
            false,
        ))
        .into()
    } else {
        Redirect::to("/personas/create").into()
    };

    drop(user);
    drop(i18n);
    drop(_db);

    res
}

#[get("/", rank = 2)]
pub fn home_redirect() -> Redirect {
    Redirect::to("/auth/sign_in")
}

//
// These are specific routes for static asset folders
// ideally they will be handled by Nginx/Apache/Web server
// but for development purposes we can handle them in Rocket :D
//

// Web root
#[cfg(debug_assertions)]
#[get("/web/<file..>")]
pub fn webroot(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("dist/").join(file);
    NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

#[cfg(debug_assertions)]
#[get("/images/<file..>")]
pub fn images(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("web/images/").join(file);
    NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

// Stylesheets root
#[cfg(debug_assertions)]
#[get("/stylesheets/<file..>")]
pub fn stylesheets(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("web/stylesheets/").join(file);
    NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

// Emoji folder
#[cfg(debug_assertions)]
#[get("/emoji/<file..>")]
pub fn emoji(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("emoji/").join(file);
    NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}

// Themes folder
#[cfg(debug_assertions)]
#[get("/themes/<file..>")]
pub fn themes(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("web/themes/").join(file);
    NamedFile::open(&path).map_err(|_| NotFound(format!("Bad path: {:?}", path)))
}
