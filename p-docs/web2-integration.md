# Web2 Integration

This guide covers end-to-end integration for:

- Facebook / Instagram donation widget
- YouTube Tip in PeaceCoin
- Telegram/Discord tip bots

All features are exposed by the `p-project-api` service. Use environment variables to configure API keys and base URL.

## Prerequisites

- API server running and reachable (e.g., `http://localhost:3000` or behind NGINX `/api`).
- JWT configured for protected endpoints if using the donation widget on authenticated pages.
- Optional platform keys set via environment variables:
  - `WEB2_FACEBOOK_KEY`, `WEB2_YOUTUBE_KEY`, `WEB2_TELEGRAM_KEY`, `WEB2_DISCORD_KEY`
  - `WEB2_WEBHOOK_URL` set to your API base (e.g., `https://example.com/api`)

## 1) Facebook / Instagram Donation Widget

Create the widget (admin-only):

```
POST /web2/create-donation-widget
Authorization: Bearer <admin_jwt>
{
  "config": {
    "platform": "facebook",       
    "page_id": "<page_or_profile_id>",
    "campaign_name": "Save the Children",
    "target_amount": 1000.0,
    "current_amount": 0.0,
    "currency": "P",
    "button_text": "Donate Now",
    "description": "Help us provide education to children in need"
  }
}
```

Generate embeddable HTML (user or admin):

```
POST /web2/generate-widget-html
Authorization: Bearer <jwt>
{
  "widget_id": "<returned widget_id>"
}
```

Embed the returned HTML on your site and include the helper script to wire up behavior:

```
<script src="/donation-widget.js"></script>
```

The widget renders buttons and a form, and posts donations to `WEB2_WEBHOOK_URL + /web2/process-social-donation` if configured, else `/web2/process-social-donation` relative to the page. If your site can inject a JWT, add `data-jwt="<short-lived JWT>"` onto the outer `<div class="p-coin-donation-widget" ...>` to authorize the call.

Optional data attributes on the widget container:

- `data-api-base` — override API base URL (defaults to current origin).
- `data-platform` — set platform label (`facebook`, `instagram`). Defaults to `facebook`.
- `data-currency` — override currency label, default `P`.

Endpoint called by the widget:

```
POST /web2/process-social-donation
Authorization: Bearer <jwt>
{
  "donation_data": {
    "widget_id": "...",
    "donor_name": "Anonymous",
    "donor_email": null,
    "amount": 10.0,
    "currency": "P",
    "platform": "facebook",
    "platform_user_id": "",
    "message": "Great work!"
  }
}
```

Response:

```
{
  "donation_response": {
    "donation_id": "donation_...",
    "success": true,
    "message": "Successfully processed donation of ...",
    "transaction_hash": "0x..."
  }
}
```

Note: The reference implementation uses in-memory storage; persist widgets in your DB if needed.

## 2) YouTube Tip in PeaceCoin

Create a YouTube Tip config (admin-only):

```
POST /web2/create-youtube-tip-config
Authorization: Bearer <admin_jwt>
{
  "config": {
    "channel_id": "UC...",
    "video_id": "dQw4w9WgXcQ",
    "default_amounts": [1, 5, 10, 25],
    "currency": "P",
    "message": "Thanks for supporting our content!"
  }
}
```

Process a tip (user/admin):

```
POST /web2/process-youtube-tip
Authorization: Bearer <jwt>
{
  "tip_data": {
    "channel_id": "UC...",
    "video_id": "dQw4w9WgXcQ",
    "tipper_name": "Jane",
    "tipper_email": "jane@example.com",
    "amount": 10.0,
    "currency": "P",
    "message": "Love your content!"
  }
}
```

Returns a standard donation response with `donation_id` prefixed by `tip_`.

## 3) Telegram/Discord Tip Bots

Register bot commands (admin-only, optional):

```
POST /web2/register-messaging-bot
Authorization: Bearer <admin_jwt>
{
  "config": {
    "platform": "telegram",
    "bot_token": "<bot_token>",
    "commands": ["tip", "help", "balance"],
    "default_tip_amount": 5.0,
    "currency": "P"
  }
}
```

Webhook setup with shared token gates (public routes):

- Telegram webhook: `POST /web2/telegram/webhook?token=<WEB2_TELEGRAM_KEY>`
- Discord webhook: `POST /web2/discord/webhook?token=<WEB2_DISCORD_KEY>`

Set `WEB2_TELEGRAM_KEY` and/or `WEB2_DISCORD_KEY` in the API environment. The webhook expects messages such as:

- Telegram: `/tip 10` or `/donate 5` in `message.text`
- Discord: `!tip 10` or `/tip 10` in `content`

On receipt, the API translates the command to a `process_bot_command` call and returns JSON with `response_text` and optional `tip_transaction_id`.

## Environment Variables

- `WEB2_WEBHOOK_URL` — Base URL for client widgets to call the API (e.g., `https://yourdomain.com/api`).
- `WEB2_FACEBOOK_KEY`, `WEB2_YOUTUBE_KEY` — Optional platform keys.
- `WEB2_TELEGRAM_KEY`, `WEB2_DISCORD_KEY` — Shared secrets for webhook verification.

## Security Notes

- The donation widget calls a protected endpoint; embed a short-lived JWT in the page (e.g., server-side rendered attribute) and set it on the widget as `data-jwt`.
- For public webhooks, a shared token gate is used. For production, prefer platform-native request signing or IP allowlists in addition to the shared token.

## NGINX

Serve the site behind NGINX (provided). Static assets are under `/usr/share/nginx/html`. If you host your own widget JavaScript, ensure Content-Security-Policy allows it (`script-src 'self'`).
