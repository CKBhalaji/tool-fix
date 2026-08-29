//! Role/authorization helpers. The backend is the only authority for roles.

use toolfix_contracts::UserRole;

use crate::error::AuthError;
use crate::middleware::AuthUser;

pub fn ensure_role(user: &AuthUser, required: UserRole) -> Result<(), AuthError> {
    if user.role == required {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

pub fn ensure_any_role(user: &AuthUser, required: &[UserRole]) -> Result<(), AuthError> {
    if required.contains(&user.role) {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}
