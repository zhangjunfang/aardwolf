use aardwolf_models::{
    base_actor::persona::Persona,
    user::{AuthenticatedUser, PermissionError, PermissionedUser, PersonaDeleter},
};
use diesel::{pg::PgConnection, result::Error as DieselError};

use crate::{
    error::AardwolfFail,
    forms::{personas::FetchPersonaFail, traits::DbAction},
};

pub struct CheckDeletePersonaPermission(AuthenticatedUser);

impl CheckDeletePersonaPermission {
    pub fn new(user: AuthenticatedUser) -> Self {
        CheckDeletePersonaPermission(user)
    }

    pub fn with(self, persona: Persona) -> CheckDeletePersonaPermissionOperation {
        CheckDeletePersonaPermissionOperation(self.0, persona)
    }
}

pub struct CheckDeletePersonaPermissionOperation(AuthenticatedUser, Persona);

impl DbAction<PersonaDeleter, CheckDeletePersonaPermissionFail>
    for CheckDeletePersonaPermissionOperation
{
    fn db_action(
        self,
        conn: &PgConnection,
    ) -> Result<PersonaDeleter, CheckDeletePersonaPermissionFail> {
        Ok(self.0.can_delete_persona(self.1, conn)?)
    }
}

#[derive(Debug, Clone, Fail, Serialize)]
pub enum CheckDeletePersonaPermissionFail {
    #[fail(display = "User does not have permission to delete persona")]
    Permission,
    #[fail(display = "Error accessing database to check permissions")]
    Database,
}

impl From<PermissionError> for CheckDeletePersonaPermissionFail {
    fn from(e: PermissionError) -> Self {
        match e {
            PermissionError::Diesel => CheckDeletePersonaPermissionFail::Database,
            PermissionError::Permission => CheckDeletePersonaPermissionFail::Permission,
        }
    }
}

impl AardwolfFail for CheckDeletePersonaPermissionFail {}

pub struct DeletePersona;

impl DeletePersona {
    pub fn with(self, persona_deleter: PersonaDeleter) -> Delete {
        Delete(persona_deleter)
    }
}

pub struct Delete(PersonaDeleter);

impl DbAction<(), PersonaDeletionFail> for Delete {
    fn db_action(self, conn: &PgConnection) -> Result<(), PersonaDeletionFail> {
        self.0.delete_persona(conn).map_err(From::from)
    }
}

#[derive(Clone, Debug, Fail, Serialize)]
pub enum PersonaDeletionFail {
    #[fail(display = "Insufficient permissions")]
    Permission,
    #[fail(display = "Error in database")]
    Database,
    #[fail(display = "Persona not found")]
    NotFound,
}

impl From<DieselError> for PersonaDeletionFail {
    fn from(e: DieselError) -> Self {
        match e {
            DieselError::NotFound => PersonaDeletionFail::NotFound,
            _ => PersonaDeletionFail::Database,
        }
    }
}

impl From<CheckDeletePersonaPermissionFail> for PersonaDeletionFail {
    fn from(e: CheckDeletePersonaPermissionFail) -> Self {
        match e {
            CheckDeletePersonaPermissionFail::Permission => PersonaDeletionFail::Permission,
            CheckDeletePersonaPermissionFail::Database => PersonaDeletionFail::Database,
        }
    }
}

impl From<FetchPersonaFail> for PersonaDeletionFail {
    fn from(e: FetchPersonaFail) -> Self {
        match e {
            FetchPersonaFail::Database => PersonaDeletionFail::Database,
            FetchPersonaFail::NotFound => PersonaDeletionFail::NotFound,
        }
    }
}

impl AardwolfFail for PersonaDeletionFail {}