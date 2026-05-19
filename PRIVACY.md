# Privacy Policy - Together

**Effective date:** 2026-05-20
**Maintainer:** Salossov &lt;aplyt228@gmail.com&gt;

**Together** is a personal, non-commercial authentication backend for
a small private Minecraft: Java Edition server (NeoForge 1.21.1).
This document describes what data the application processes, why,
and how it is stored.

The application is **not** a public service. Access is limited to a
group of personal friends. There is no monetization, advertising, or
data sharing with third parties.

## What data we collect

### Local accounts (username + password)

- **Username** - chosen by the player, used as their in-game name
- **Email** *(optional)* - for password recovery only; never sent
  marketing or third-party communications
- **Password hash** - bcrypt with the default cost factor; the
  plaintext password is never stored or logged
- **Permanent UUID** - generated server-side (UUIDv4), used as the
  stable identifier inside the modded server

### Microsoft sign-in

When a player chooses to sign in via Microsoft, the backend performs
the standard Minecraft authentication chain:

1. Exchange `code` for a Microsoft `access_token`
   (`login.live.com/oauth20_token.srf`)
2. Exchange `access_token` for an Xbox Live token
   (`user.auth.xboxlive.com/user/authenticate`)
3. Exchange XBL token for an XSTS token
   (`xsts.auth.xboxlive.com/xsts/authorize`)
4. Exchange XSTS for a Minecraft access token
   (`api.minecraftservices.com/authentication/login_with_xbox`)
5. Fetch the player profile
   (`api.minecraftservices.com/minecraft/profile`)
6. Fetch the signed skin texture
   (`sessionserver.mojang.com/session/minecraft/profile/{uuid}`)

From this chain we **persist** in our database:

- The player's Mojang **UUID** (used as their permanent identifier)
- The player's **in-game username**
- The **base64-encoded skin texture** and its **Mojang signature**
- A flag indicating the account type is `MICROSOFT`

We **temporarily process but do not store**:

- Microsoft access tokens, Xbox Live tokens, XSTS tokens, Minecraft
  access tokens - discarded as soon as the chain completes
- Microsoft refresh tokens - we do not request `offline_access`
  storage; refresh is not used
- Email and Xbox gamertag - only displayed back to the player in
  error messages; **not** stored in the database

## What we do NOT do

- We do **not** sell, share, or monetize player data
- We do **not** use any analytics, telemetry, or tracking
- We do **not** integrate with advertising networks
- We do **not** keep Microsoft/Mojang tokens after the auth chain finishes
- We do **not** read messages, friend lists, or any Xbox social data;
  the only Xbox API call is `profile/settings?settings=Gamertag` to
  improve error messages

## How data is stored

- The database is self-hosted (SQLite, PostgreSQL or MySQL - operator's
  choice via `DATABASE_URL`).
- The optional `BACKUP_DATABASE_URL` mirrors writes to a second
  database for redundancy. It receives the same data described above
  and the same privacy guarantees apply.
- All HTTP traffic between the frontend and backend is expected to be
  served over HTTPS in production.

## Session management

After a successful sign-in the backend issues a **JWT** (HS256, 30-day
TTL) containing only:

- `sub` - the player's permanent UUID
- `username` - the player's in-game name
- `auth_type` - `LOCAL` or `MICROSOFT`
- `iat`, `exp` - standard JWT timestamps

The signing secret is kept server-side and never exposed to the browser.

## Data retention and deletion

- Account data is kept while the user has an active account.
- A player can request full deletion at any time by contacting the
  maintainer at the address below. Deletion is performed within 30 days.

## Microsoft Identity Platform compliance

This application uses the Microsoft Identity Platform via Azure AD as
a confidential OAuth 2.0 client:

- All token exchanges are server-side only, over HTTPS
- The `client_secret` is stored exclusively on the server
- We follow the standard Authorization Code flow as documented at
  <https://wiki.vg/Microsoft_Authentication_Scheme>
- The application does not require any non-default Microsoft Graph
  permissions; only `XboxLive.signin offline_access openid email`

## Contact

For questions about this policy, data deletion requests, or anything
else: **aplyt228@gmail.com**
