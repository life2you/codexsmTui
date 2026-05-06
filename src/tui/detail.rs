use anyhow::Result;

use crate::codex::{
    parser::load_session_detail,
    session::{Session, SessionDetail},
};

pub fn load_detail(session: &Session) -> Result<SessionDetail> {
    load_session_detail(session)
}
