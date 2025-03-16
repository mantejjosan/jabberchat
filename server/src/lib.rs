use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: Option<String>,
    online: bool,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String
}

#[reducer]
/// Clients invoke this reducer to sset their usernames
pub fn set_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    let name = validate_name(name)?;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User { name: Some(name), ..user});
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

/// Takes a name and checks if it is acceptable as a username
pub fn validate_name(name: String) -> Result<String, String> {
    if name.is_empty() {
        Err("Name must not be empty".to_string())
    } else {
        Ok(name)
    }
}

#[reducer]
pub fn send_message(ctx: &ReducerContext, msg: String) -> Result<(), String> {
    let msg = validate_message(msg)?;
    log::info!("{}", msg);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        sent: ctx.timestamp,
        text: msg,
    });
    Ok(())
}

fn validate_message(text: String) -> Result<String, String> {
    if text.is_empty() {
        Err("Message can't be empty!".to_string())
    } else {
        Ok(text)
    }
}

#[reducer(client_connected)]
// called when a client connects to the spacetime database
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        // If this is a returning user, meaning we already have a user with this Identityset
        // online: true, but leave the name and Identity unchanged
        ctx.db.user().identity().update(User { online: true, ..user });
    } else {
        // If this is a new user, create a User row with the Identity,
        // which is online, but hasn't set a name
        ctx.db.user().insert(User {
            identity: ctx.sender,
            name: None,
            online: true,
        });
    }
}
 #[reducer(client_disconnected)]
// Called when a client disconnects
pub fn client_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User { online: false, ..user });
    } else {
        // This branch should be unreachable as it doesn't make sense for the uset to 
        // disconnect without connceting first
        log::warn!("Disconnect event for unknown user with the identity {:?}", ctx.sender);
    }

}

