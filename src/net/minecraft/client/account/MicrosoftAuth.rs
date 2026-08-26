use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::{json, Value};
use thiserror::Error;
use url::Url;

use crate::net::minecraft::util::Session::Session;

pub const CLIENT_ID: &str = "42a60a84-599d-44b2-a7c6-b00cdef1d6a2";
pub const CALLBACK_PORT: u16 = 25_575;
const AUTHORIZE_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const XBOX_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MINECRAFT_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MINECRAFT_PROFILE_NAME_URL: &str =
    "https://api.minecraftservices.com/minecraft/profile/name/";
const LEGACY_AUTHORIZE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const LEGACY_CLIENT_ID: &str = "000000004C12AE6F";
const LEGACY_REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
const LEGACY_SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";
const TOKEN_LOGIN_CLIENT_ID: &str = "00000000402b5328";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrosoftLogin {
    pub session: Session,
    pub refreshToken: String,
    pub accessToken: String,
}

#[derive(Debug, Error)]
pub enum MicrosoftAuthError {
    #[error("failed starting OAuth callback server: {0}")]
    CallbackServer(#[source] std::io::Error),
    #[error("failed opening the system browser")]
    Browser,
    #[error("invalid OAuth callback: {0}")]
    Callback(String),
    #[error("Microsoft authentication request failed: {0}")]
    Http(String),
    #[error("Microsoft authentication response was missing {0}")]
    Missing(&'static str),
    #[error("Microsoft authentication response was malformed: {0}")]
    Malformed(String),
    #[error("Microsoft authentication was cancelled")]
    Cancelled,
    #[error("Microsoft authentication timed out waiting for the browser callback")]
    TimedOut,
}

pub fn interactive_login(
    status: Option<&Sender<String>>,
    cancelled: Option<&AtomicBool>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fCheck your browser to continue...§r");
    let code = acquire_authorization_code(cancelled)?;
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fAcquiring Microsoft access tokens§r");
    let tokens = exchange_authorization_code(&code)?;
    complete_microsoft_login(tokens.0, tokens.1, status, cancelled)
}

/// Rust equivalent of Exhibition's external `openauth-1.1.6` credential branch.
///
/// The original client performs the legacy Microsoft form exchange locally,
/// never persists the password, and then enters the same Xbox/XSTS/Minecraft
/// token chain as the browser login. Microsoft may reject this flow for
/// accounts that require two-factor confirmation or additional approval.
pub fn login_with_credentials(
    username: &str,
    password: &str,
    status: Option<&Sender<String>>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    if username.trim().is_empty() {
        return Err(MicrosoftAuthError::Missing("Microsoft account name"));
    }
    if password.is_empty() {
        return Err(MicrosoftAuthError::Missing("Microsoft account password"));
    }
    send_status(status, "§eLogging in...§r");

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(30))
        .timeout_write(Duration::from_secs(30))
        .redirects(10)
        .build();
    let loginPage = request_text(
        agent
            .get(LEGACY_AUTHORIZE_URL)
            .query("client_id", LEGACY_CLIENT_ID)
            .query("redirect_uri", LEGACY_REDIRECT_URI)
            .query("scope", LEGACY_SCOPE)
            .query("response_type", "token")
            .query("display", "touch")
            .query("locale", "en")
            .call(),
    )?;
    let ppft = extract_between(&loginPage, "sFTTag:'", "value=\"")
        .and_then(|tail| tail.split('\"').next())
        .filter(|value| !value.is_empty())
        .ok_or(MicrosoftAuthError::Missing("Microsoft PPFT form token"))?;
    let urlPost = extract_js_string(&loginPage, "urlPost:")
        .ok_or(MicrosoftAuthError::Missing("Microsoft credential form URL"))?
        .replace("&amp;", "&");

    let response = agent
        .post(&urlPost)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_form(&[
            ("login", username),
            ("loginfmt", username),
            ("passwd", password),
            ("PPFT", ppft),
        ]);
    let (finalUrl, body) = match response {
        Ok(response) => {
            let finalUrl = response.get_url().to_owned();
            let body = response.into_string().unwrap_or_default();
            (finalUrl, body)
        }
        Err(ureq::Error::Status(statusCode, response)) => {
            let finalUrl = response.get_url().to_owned();
            let body = response.into_string().unwrap_or_default();
            if finalUrl.contains("access_token=") {
                (finalUrl, body)
            } else {
                return Err(MicrosoftAuthError::Http(format!(
                    "HTTP {statusCode}: {}",
                    body.trim()
                )));
            }
        }
        Err(ureq::Error::Transport(error)) => {
            return Err(MicrosoftAuthError::Http(error.to_string()))
        }
    };
    if body.contains("identity/confirm") {
        return Err(MicrosoftAuthError::Http(
            "User has enabled double-authentication or must allow sign-in on https://account.live.com/activity".to_owned(),
        ));
    }
    let microsoftAccessToken = extract_url_value(&finalUrl, "access_token")
        .ok_or_else(|| MicrosoftAuthError::Http("Invalid credentials or tokens".to_owned()))?;
    let refreshToken = extract_url_value(&finalUrl, "refresh_token").unwrap_or_default();

    // openauth-1.1.6 sends its implicit-flow token without the modern `d=`
    // prefix. Keep this branch separate from Exhibition's browser OAuth path.
    send_status(status, "§fAcquiring Xbox access token§r");
    let xboxToken = acquire_xbox_access_token_internal(&microsoftAccessToken, false)?;
    send_status(status, "§fAcquiring Xbox XSTS token§r");
    let (xstsToken, userHash) = acquire_xsts_token(&xboxToken)?;
    send_status(status, "§fAcquiring Minecraft access token§r");
    let minecraftAccessToken = acquire_minecraft_access_token(&xstsToken, &userHash)?;
    send_status(status, "§fChecking Minecraft ownership§r");
    ensure_minecraft_entitlement(&minecraftAccessToken)?;
    send_status(status, "§fFetching your Minecraft profile§r");
    let profile = login_with_minecraft_access_token(&minecraftAccessToken)?;
    let session = Session::new(
        profile.getUsername(),
        profile.getPlayerID(),
        minecraftAccessToken.clone(),
        "microsoft",
    );
    Ok(MicrosoftLogin {
        session,
        refreshToken,
        accessToken: minecraftAccessToken,
    })
}

pub fn refresh_login(
    refreshToken: &str,
    status: Option<&Sender<String>>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    refresh_login_cancelable(refreshToken, status, None)
}

fn refresh_login_cancelable(
    refreshToken: &str,
    status: Option<&Sender<String>>,
    cancelled: Option<&AtomicBool>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fRefreshing Microsoft access tokens§r");
    let (microsoftAccessToken, refreshToken) = refresh_microsoft_tokens(refreshToken)?;
    complete_microsoft_login(microsoftAccessToken, refreshToken, status, cancelled)
}

pub fn login_saved_account(
    accessToken: &str,
    refreshToken: &str,
    status: Option<&Sender<String>>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    login_saved_account_cancelable(accessToken, refreshToken, status, None)
}

pub fn login_saved_account_cancelable(
    accessToken: &str,
    refreshToken: &str,
    status: Option<&Sender<String>>,
    cancelled: Option<&AtomicBool>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    ensure_not_cancelled(cancelled)?;
    if !accessToken.trim().is_empty() {
        send_status(status, "§fFetching your Minecraft profile§r");
        if let Ok(session) = login_with_minecraft_access_token(accessToken) {
            ensure_not_cancelled(cancelled)?;
            return Ok(MicrosoftLogin {
                session,
                refreshToken: refreshToken.to_owned(),
                accessToken: accessToken.to_owned(),
            });
        }
    }
    ensure_not_cancelled(cancelled)?;
    if refreshToken.trim().is_empty() {
        return Err(MicrosoftAuthError::Http(
            "access token is invalid or expired and no refresh token is available".to_owned(),
        ));
    }
    refresh_login_cancelable(refreshToken, status, cancelled)
}

pub fn token_login(
    token: &str,
    status: Option<&Sender<String>>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    if token.starts_with("M.C") {
        refresh_token_login(token, status)
    } else {
        send_status(status, "§fFetching your Minecraft profile§r");
        let session = login_with_minecraft_access_token(token)?;
        Ok(MicrosoftLogin {
            session,
            refreshToken: String::new(),
            accessToken: token.to_owned(),
        })
    }
}

fn refresh_token_login(
    refreshToken: &str,
    status: Option<&Sender<String>>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    send_status(status, "§fRefreshing Microsoft access tokens§r");
    let value = request_json(
        ureq::post(TOKEN_URL)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .timeout(Duration::from_secs(30))
            .send_form(&[
                ("client_id", TOKEN_LOGIN_CLIENT_ID),
                ("scope", LEGACY_SCOPE),
                ("grant_type", "refresh_token"),
                ("redirect_uri", LEGACY_REDIRECT_URI),
                ("refresh_token", refreshToken),
            ]),
    )?;
    let microsoftAccessToken = json_string(&value, "access_token")?;
    let refreshToken = json_string(&value, "refresh_token")?;

    send_status(status, "§fAcquiring Xbox access token§r");
    let xboxToken = acquire_xbox_access_token_internal(&microsoftAccessToken, false)?;
    send_status(status, "§fAcquiring Xbox XSTS token§r");
    let (xstsToken, userHash) = acquire_xsts_token(&xboxToken)?;
    send_status(status, "§fAcquiring Minecraft access token§r");
    let minecraftAccessToken = acquire_minecraft_access_token(&xstsToken, &userHash)?;
    send_status(status, "§fFetching your Minecraft profile§r");
    let (session, minecraftAccessToken) =
        login_or_create_token_profile(minecraftAccessToken, &xstsToken, &userHash, status)?;
    Ok(MicrosoftLogin {
        session,
        refreshToken,
        accessToken: minecraftAccessToken,
    })
}

fn login_or_create_token_profile(
    minecraftAccessToken: String,
    xstsToken: &str,
    userHash: &str,
    status: Option<&Sender<String>>,
) -> Result<(Session, String), MicrosoftAuthError> {
    if let Some(session) = try_login_with_minecraft_access_token(&minecraftAccessToken)? {
        return Ok((session, minecraftAccessToken));
    }

    ensure_minecraft_entitlement(&minecraftAccessToken)?;
    let name = prompt_for_minecraft_profile_name()?;
    send_status(status, "§fCreating your Minecraft profile§r");
    create_minecraft_profile_name(&minecraftAccessToken, &name)?;

    // Exhibition requests a new Minecraft access token after profile creation
    // before reading the profile again. Preserve that sequence rather than
    // assuming the previous token has immediately observed the new profile.
    send_status(status, "§fAcquiring Minecraft access token§r");
    let refreshedMinecraftAccessToken = acquire_minecraft_access_token(xstsToken, userHash)?;
    send_status(status, "§fFetching your Minecraft profile§r");
    let session = login_with_minecraft_access_token(&refreshedMinecraftAccessToken)?;
    Ok((session, refreshedMinecraftAccessToken))
}

fn try_login_with_minecraft_access_token(
    token: &str,
) -> Result<Option<Session>, MicrosoftAuthError> {
    if token.trim().is_empty() {
        return Err(MicrosoftAuthError::Missing("Minecraft access token"));
    }
    let request = ureq::get(MINECRAFT_PROFILE_URL)
        .set("Authorization", &format!("Bearer {token}"))
        .timeout(Duration::from_secs(30))
        .call();
    let value = match request {
        Ok(response) => response
            .into_json::<Value>()
            .map_err(|error| MicrosoftAuthError::Malformed(error.to_string()))?,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(error) => return request_json(Err(error)).map(|_| None),
    };
    let username = json_string(&value, "name")?;
    let uuid = json_string(&value, "id")?;
    Ok(Some(Session::new(username, uuid, token, "mojang")))
}

fn ensure_minecraft_entitlement(token: &str) -> Result<(), MicrosoftAuthError> {
    let value = request_json(
        ureq::get(MINECRAFT_ENTITLEMENTS_URL)
            .set("Authorization", &format!("Bearer {token}"))
            .timeout(Duration::from_secs(30))
            .call(),
    )?;
    let items = value
        .get("items")
        .and_then(Value::as_array)
        .ok_or(MicrosoftAuthError::Missing("Minecraft entitlement items"))?;
    let hasMinecraft = items.iter().any(|item| {
        matches!(
            item.get("name").and_then(Value::as_str),
            Some("product_minecraft" | "game_minecraft")
        )
    });
    if !hasMinecraft {
        return Err(MicrosoftAuthError::Http(
            "This Microsoft account dont have minecraft.".to_owned(),
        ));
    }
    Ok(())
}

fn create_minecraft_profile_name(token: &str, name: &str) -> Result<(), MicrosoftAuthError> {
    let url = minecraft_profile_name_url(name)?;
    let result = ureq::put(url.as_str())
        .set("Accept", "*/*")
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "MojangSharp/0.1")
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(30))
        .call();
    match result {
        Ok(response) if matches!(response.status(), 200 | 204) => Ok(()),
        Ok(response) => Err(profile_name_status_error(response.status())),
        Err(ureq::Error::Status(status, _)) => Err(profile_name_status_error(status)),
        Err(ureq::Error::Transport(error)) => Err(MicrosoftAuthError::Http(error.to_string())),
    }
}

fn minecraft_profile_name_url(name: &str) -> Result<Url, MicrosoftAuthError> {
    let mut url = Url::parse(MINECRAFT_PROFILE_NAME_URL)
        .map_err(|error| MicrosoftAuthError::Malformed(error.to_string()))?;
    url.path_segments_mut()
        .map_err(|_| {
            MicrosoftAuthError::Malformed("invalid Minecraft profile name URL".to_owned())
        })?
        .push(name);
    Ok(url)
}

fn profile_name_status_error(status: u16) -> MicrosoftAuthError {
    let cause = match status {
        400 => "Name is invaild",
        403 => "Name is unlivable",
        401 => "Unauthorized",
        429 => "Too many requests",
        500 => "Mojang API lags",
        _ => "Unknown",
    };
    MicrosoftAuthError::Http(format!("Failed to change name due to {cause}"))
}

fn prompt_for_minecraft_profile_name() -> Result<String, MicrosoftAuthError> {
    #[cfg(target_os = "windows")]
    {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$form = New-Object System.Windows.Forms.Form
$form.Text = 'Minecraft Profile'
$form.StartPosition = 'CenterScreen'
$form.TopMost = $true
$form.FormBorderStyle = 'FixedDialog'
$form.MaximizeBox = $false
$form.MinimizeBox = $false
$form.ClientSize = New-Object System.Drawing.Size(430, 115)
$label = New-Object System.Windows.Forms.Label
$label.Text = 'No minecraft profile found, please set a new name.'
$label.AutoSize = $true
$label.Location = New-Object System.Drawing.Point(12, 12)
$text = New-Object System.Windows.Forms.TextBox
$text.Location = New-Object System.Drawing.Point(12, 38)
$text.Size = New-Object System.Drawing.Size(406, 23)
$ok = New-Object System.Windows.Forms.Button
$ok.Text = 'OK'
$ok.DialogResult = [System.Windows.Forms.DialogResult]::OK
$ok.Location = New-Object System.Drawing.Point(262, 76)
$cancel = New-Object System.Windows.Forms.Button
$cancel.Text = 'Cancel'
$cancel.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
$cancel.Location = New-Object System.Drawing.Point(343, 76)
$form.Controls.AddRange(@($label, $text, $ok, $cancel))
$form.AcceptButton = $ok
$form.CancelButton = $cancel
$form.Add_Shown({ $text.Focus() })
$result = $form.ShowDialog()
if ($result -ne [System.Windows.Forms.DialogResult]::OK) { exit 2 }
[Console]::Write($text.Text)
"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .map_err(|error| {
                MicrosoftAuthError::Http(format!("Failed opening profile name dialog: {error}"))
            })?;
        if output.status.code() == Some(2) {
            return Err(MicrosoftAuthError::Http(
                "Minecraft name is null".to_owned(),
            ));
        }
        if !output.status.success() {
            return Err(MicrosoftAuthError::Http(
                "Failed to change name.".to_owned(),
            ));
        }
        let name = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if name.is_empty() {
            return Err(MicrosoftAuthError::Http(
                "Minecraft name is null".to_owned(),
            ));
        }
        Ok(name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(MicrosoftAuthError::Http(
            "Minecraft name is null".to_owned(),
        ))
    }
}

pub fn login_with_minecraft_access_token(token: &str) -> Result<Session, MicrosoftAuthError> {
    if token.trim().is_empty() {
        return Err(MicrosoftAuthError::Missing("Minecraft access token"));
    }
    let value = request_json(
        ureq::get(MINECRAFT_PROFILE_URL)
            .set("Authorization", &format!("Bearer {token}"))
            .timeout(Duration::from_secs(30))
            .call(),
    )?;
    let username = json_string(&value, "name")?;
    let uuid = json_string(&value, "id")?;
    Ok(Session::new(username, uuid, token, "mojang"))
}

fn complete_microsoft_login(
    microsoftAccessToken: String,
    refreshToken: String,
    status: Option<&Sender<String>>,
    cancelled: Option<&AtomicBool>,
) -> Result<MicrosoftLogin, MicrosoftAuthError> {
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fAcquiring Xbox access token§r");
    let xboxToken = acquire_xbox_access_token(&microsoftAccessToken)?;
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fAcquiring Xbox XSTS token§r");
    let (xstsToken, userHash) = acquire_xsts_token(&xboxToken)?;
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fAcquiring Minecraft access token§r");
    let minecraftAccessToken = acquire_minecraft_access_token(&xstsToken, &userHash)?;
    ensure_not_cancelled(cancelled)?;
    send_status(status, "§fFetching your Minecraft profile§r");
    let session = login_with_minecraft_access_token(&minecraftAccessToken)?;
    ensure_not_cancelled(cancelled)?;
    Ok(MicrosoftLogin {
        session,
        refreshToken,
        accessToken: minecraftAccessToken,
    })
}

fn build_authorization_url(state: &str) -> Url {
    let redirectUri = format!("http://localhost:{CALLBACK_PORT}/callback");
    let mut authorize = Url::parse(AUTHORIZE_URL).expect("fixed Microsoft authorize URL");
    authorize
        .query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirectUri)
        .append_pair("scope", "XboxLive.signin XboxLive.offline_access")
        .append_pair("state", state)
        .append_pair("prompt", "select_account");
    authorize
}

fn oauth_callback_error(
    errorCode: Option<String>,
    errorDescription: Option<String>,
) -> Option<String> {
    match (errorCode, errorDescription) {
        (Some(code), Some(description)) => Some(format!("{code}: {description}")),
        (Some(code), None) => Some(code),
        (None, Some(description)) => Some(description),
        (None, None) => None,
    }
}

fn acquire_authorization_code(
    cancelled: Option<&AtomicBool>,
) -> Result<String, MicrosoftAuthError> {
    let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT))
        .map_err(MicrosoftAuthError::CallbackServer)?;
    listener
        .set_nonblocking(true)
        .map_err(MicrosoftAuthError::CallbackServer)?;
    let state: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let authorize = build_authorization_url(&state);
    open_browser(authorize.as_str())?;

    let deadline = Instant::now() + Duration::from_secs(5 * 60);
    let (mut stream, _) = loop {
        ensure_not_cancelled(cancelled)?;
        if Instant::now() >= deadline {
            return Err(MicrosoftAuthError::TimedOut);
        }
        match listener.accept() {
            Ok(connection) => break connection,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(MicrosoftAuthError::CallbackServer(error)),
        }
    };
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
    let target = read_request_target(&mut stream)?;
    let callback = Url::parse(&format!("http://localhost:{CALLBACK_PORT}{target}"))
        .map_err(|error| MicrosoftAuthError::Callback(error.to_string()))?;
    let mut code = None;
    let mut returnedState = None;
    let mut errorCode = None;
    let mut errorDescription = None;
    for (name, value) in callback.query_pairs() {
        match name.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => returnedState = Some(value.into_owned()),
            "error" => errorCode = Some(value.into_owned()),
            "error_description" => errorDescription = Some(value.into_owned()),
            _ => {}
        }
    }

    let stateMatches = returnedState.as_deref() == Some(state.as_str());
    let success = stateMatches && code.is_some() && errorCode.is_none();
    write_callback_page(&mut stream, success);

    // A successful authorization response must match the nonce before its code
    // can enter the token exchange.  Error callbacks do not carry credentials,
    // so preserve Microsoft's actual diagnostic instead of masking it with a
    // secondary state error when the request itself was rejected.
    if let Some(code) = code {
        if !stateMatches {
            return Err(MicrosoftAuthError::Callback(
                "OAuth state mismatch".to_owned(),
            ));
        }
        return Ok(code);
    }
    if let Some(error) = oauth_callback_error(errorCode, errorDescription) {
        return Err(MicrosoftAuthError::Callback(error));
    }
    if !stateMatches {
        return Err(MicrosoftAuthError::Callback(
            "OAuth state mismatch".to_owned(),
        ));
    }
    Err(MicrosoftAuthError::Callback(
        "no authorization code was returned".to_owned(),
    ))
}

fn ensure_not_cancelled(cancelled: Option<&AtomicBool>) -> Result<(), MicrosoftAuthError> {
    if cancelled.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        Err(MicrosoftAuthError::Cancelled)
    } else {
        Ok(())
    }
}

fn exchange_authorization_code(code: &str) -> Result<(String, String), MicrosoftAuthError> {
    let redirectUri = format!("http://localhost:{CALLBACK_PORT}/callback");
    let value = request_json(
        ureq::post(TOKEN_URL)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .timeout(Duration::from_secs(30))
            .send_form(&[
                ("client_id", CLIENT_ID),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirectUri.as_str()),
            ]),
    )?;
    Ok((
        json_string(&value, "access_token")?,
        json_string(&value, "refresh_token")?,
    ))
}

fn refresh_microsoft_tokens(refreshToken: &str) -> Result<(String, String), MicrosoftAuthError> {
    let redirectUri = format!("http://localhost:{CALLBACK_PORT}/callback");
    let value = request_json(
        ureq::post(TOKEN_URL)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .timeout(Duration::from_secs(30))
            .send_form(&[
                ("client_id", CLIENT_ID),
                ("grant_type", "refresh_token"),
                ("refresh_token", refreshToken),
                ("redirect_uri", redirectUri.as_str()),
            ]),
    )?;
    Ok((
        json_string(&value, "access_token")?,
        json_string(&value, "refresh_token")?,
    ))
}

fn acquire_xbox_access_token(microsoftAccessToken: &str) -> Result<String, MicrosoftAuthError> {
    acquire_xbox_access_token_internal(microsoftAccessToken, true)
}

fn acquire_xbox_access_token_internal(
    microsoftAccessToken: &str,
    prefixWithD: bool,
) -> Result<String, MicrosoftAuthError> {
    let rpsTicket = if prefixWithD {
        format!("d={microsoftAccessToken}")
    } else {
        microsoftAccessToken.to_owned()
    };
    let value = request_json(
        ureq::post(XBOX_AUTH_URL)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(30))
            .send_json(json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": rpsTicket
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            })),
    )?;
    json_string(&value, "Token")
}

fn acquire_xsts_token(xboxToken: &str) -> Result<(String, String), MicrosoftAuthError> {
    let value = request_json(
        ureq::post(XSTS_URL)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(30))
            .send_json(json!({
                "Properties": { "SandboxId": "RETAIL", "UserTokens": [xboxToken] },
                "RelyingParty": "rp://api.minecraftservices.com/",
                "TokenType": "JWT"
            })),
    )?;
    let token = json_string(&value, "Token")?;
    let userHash = value
        .get("DisplayClaims")
        .and_then(|claims| claims.get("xui"))
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(|value| value.get("uhs"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(MicrosoftAuthError::Missing("Xbox user hash"))?
        .to_owned();
    Ok((token, userHash))
}

fn acquire_minecraft_access_token(
    xstsToken: &str,
    userHash: &str,
) -> Result<String, MicrosoftAuthError> {
    let value = request_json(
        ureq::post(MINECRAFT_AUTH_URL)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(30))
            .send_json(json!({ "identityToken": format!("XBL3.0 x={userHash};{xstsToken}") })),
    )?;
    json_string(&value, "access_token")
}

fn request_text(result: Result<ureq::Response, ureq::Error>) -> Result<String, MicrosoftAuthError> {
    match result {
        Ok(response) => response
            .into_string()
            .map_err(|error| MicrosoftAuthError::Malformed(error.to_string())),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(MicrosoftAuthError::Http(if body.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                format!("HTTP {status}: {}", body.trim())
            }))
        }
        Err(ureq::Error::Transport(error)) => Err(MicrosoftAuthError::Http(error.to_string())),
    }
}

fn extract_between<'a>(source: &'a str, prefix: &str, marker: &str) -> Option<&'a str> {
    let start = source.find(prefix)? + prefix.len();
    let remainder = &source[start..];
    let markerIndex = remainder.find(marker)? + marker.len();
    Some(&remainder[markerIndex..])
}

fn extract_js_string(source: &str, key: &str) -> Option<String> {
    let start = source.find(key)? + key.len();
    let remainder = source[start..].trim_start();
    let quote = remainder.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let value = &remainder[quote.len_utf8()..];
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == quote {
            return Some(value[..index].replace("\\/", "/"));
        }
    }
    None
}

fn extract_url_value(url: &str, key: &str) -> Option<String> {
    let parameters = url
        .split_once('#')
        .map(|(_, fragment)| fragment)
        .or_else(|| url.split_once('?').map(|(_, query)| query))?;
    url::form_urlencoded::parse(parameters.as_bytes())
        .find_map(|(name, value)| (name == key).then(|| value.into_owned()))
        .filter(|value| !value.trim().is_empty())
}

fn request_json(result: Result<ureq::Response, ureq::Error>) -> Result<Value, MicrosoftAuthError> {
    match result {
        Ok(response) => response
            .into_json::<Value>()
            .map_err(|error| MicrosoftAuthError::Malformed(error.to_string())),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            let message = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|value| {
                    ["error_description", "errorMessage", "Message", "error"]
                        .into_iter()
                        .find_map(|key| {
                            value
                                .get(key)
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned)
                        })
                })
                .unwrap_or_else(|| body.trim().to_owned());
            Err(MicrosoftAuthError::Http(if message.is_empty() {
                format!("HTTP {status}")
            } else {
                format!("HTTP {status}: {message}")
            }))
        }
        Err(ureq::Error::Transport(error)) => Err(MicrosoftAuthError::Http(error.to_string())),
    }
}

fn json_string(value: &Value, key: &'static str) -> Result<String, MicrosoftAuthError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or(MicrosoftAuthError::Missing(key))
}

fn send_status(status: Option<&Sender<String>>, value: &str) {
    if let Some(status) = status {
        let _ = status.send(value.to_owned());
    }
}

fn read_request_target(stream: &mut TcpStream) -> Result<String, MicrosoftAuthError> {
    let mut bytes = Vec::with_capacity(2048);
    let mut buffer = [0u8; 512];
    loop {
        let count = stream
            .read(&mut buffer)
            .map_err(MicrosoftAuthError::CallbackServer)?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.windows(4).any(|window| window == b"\r\n\r\n") || bytes.len() > 16 * 1024 {
            break;
        }
    }
    let request = String::from_utf8_lossy(&bytes);
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .map(ToOwned::to_owned)
        .ok_or_else(|| MicrosoftAuthError::Callback("invalid HTTP callback request".to_owned()))
}

fn write_callback_page(stream: &mut TcpStream, success: bool) {
    let body = if success {
        "<!doctype html><html><body style=\"font-family:sans-serif;background:#111;color:#eee\"><h2>Microsoft authentication complete</h2><p>You may close this window and return to Minecraft.</p></body></html>"
    } else {
        "<!doctype html><html><body style=\"font-family:sans-serif;background:#111;color:#eee\"><h2>Microsoft authentication failed</h2><p>Return to Minecraft for details.</p></body></html>"
    };
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn open_browser(url: &str) -> Result<(), MicrosoftAuthError> {
    #[cfg(target_os = "windows")]
    // Do not route an OAuth URL through `cmd /C start`: `cmd.exe` treats every
    // unescaped `&` in the query string as a command separator, truncating the
    // authorization request.  rundll32 receives the URL as one process
    // argument and delegates it to the user's registered HTTPS handler.
    let status = Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .status();
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(url).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(url).status();
    status
        .ok()
        .filter(|status| status.success())
        .map(|_| ())
        .ok_or(MicrosoftAuthError::Browser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_url_uses_exhibition_client_and_callback() {
        assert_eq!(CLIENT_ID, "42a60a84-599d-44b2-a7c6-b00cdef1d6a2");
        assert_eq!(CALLBACK_PORT, 25_575);
        assert_eq!(TOKEN_LOGIN_CLIENT_ID, "00000000402b5328");

        let authorize = build_authorization_url("nonce-value");
        let query = authorize
            .query_pairs()
            .into_owned()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(query.get("client_id").map(String::as_str), Some(CLIENT_ID));
        assert_eq!(query.get("response_type").map(String::as_str), Some("code"));
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some("http://localhost:25575/callback")
        );
        assert_eq!(
            query.get("scope").map(String::as_str),
            Some("XboxLive.signin XboxLive.offline_access")
        );
        assert_eq!(query.get("state").map(String::as_str), Some("nonce-value"));
        assert_eq!(
            query.get("prompt").map(String::as_str),
            Some("select_account")
        );
    }

    #[test]
    fn oauth_error_description_is_not_masked_by_missing_state() {
        assert_eq!(
            oauth_callback_error(
                Some("invalid_request".to_owned()),
                Some("redirect_uri was missing".to_owned())
            )
            .as_deref(),
            Some("invalid_request: redirect_uri was missing")
        );
    }

    #[test]
    fn legacy_form_fields_match_openauth_page_shape() {
        let html = r#"<script>var cfg={urlPost:'https://login.live.com/ppsecure/post.srf?x=1&amp;y=2',sFTTag:'<input type="hidden" name="PPFT" value="token-value"/>'};</script>"#;
        let ppft =
            extract_between(html, "sFTTag:'", "value=\"").and_then(|tail| tail.split('\"').next());
        assert_eq!(ppft, Some("token-value"));
        assert_eq!(
            extract_js_string(html, "urlPost:").as_deref(),
            Some("https://login.live.com/ppsecure/post.srf?x=1&amp;y=2")
        );
    }

    #[test]
    fn implicit_flow_tokens_are_read_from_fragment() {
        let url =
            "https://login.live.com/oauth20_desktop.srf#access_token=a%2Bb&refresh_token=M.C.test";
        assert_eq!(
            extract_url_value(url, "access_token").as_deref(),
            Some("a+b")
        );
        assert_eq!(
            extract_url_value(url, "refresh_token").as_deref(),
            Some("M.C.test")
        );
    }

    #[test]
    fn profile_name_is_encoded_as_one_path_segment() {
        let url = minecraft_profile_name_url("Name With Space").unwrap();
        assert_eq!(
            url.as_str(),
            "https://api.minecraftservices.com/minecraft/profile/name/Name%20With%20Space"
        );
    }

    #[test]
    fn entitlement_items_accept_both_minecraft_product_names() {
        for name in ["product_minecraft", "game_minecraft"] {
            let value = json!({ "items": [{ "name": name }] });
            let has_minecraft = value
                .get("items")
                .and_then(Value::as_array)
                .unwrap()
                .iter()
                .any(|item| {
                    matches!(
                        item.get("name").and_then(Value::as_str),
                        Some("product_minecraft" | "game_minecraft")
                    )
                });
            assert!(has_minecraft);
        }
    }

    #[test]
    fn cancellation_flag_is_observed() {
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            ensure_not_cancelled(Some(&cancelled)),
            Err(MicrosoftAuthError::Cancelled)
        ));
    }
}
