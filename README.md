# BIT Agents on Aomi

One Aomi agent that runs every strategy built on BITAGENTS.

A strategy is a spec stored by the BITAGENTS strategy service. The app
has two tools, and each call names the strategy by id:

| Tool | What it does |
| --- | --- |
| `bitagents_status` | Reads the strategy's rules, limits, signals and the connected wallet's holdings. Never trades. |
| `bitagents_run` | Runs it once for the connected Solana wallet and returns orders plus a `next` field. |

The app never signs or sends anything. In paper mode the strategy service has
already simulated each swap on mainnet. In live mode the model stages the
returned instructions with Aomi's own Solana tools, and the wallet signs.

## Secrets

None are required. Both are optional overrides, set in the Project's Environment tab:

| Slot | Value |
| --- | --- |
| `BITAGENTS_API_URL` | Base URL of the strategy service, if not the default in `src/lib.rs`. |
| `BITAGENTS_APP_KEY` | Sent as `x-app-key` when the service has `BUILDER_APP_KEY` set. |

## Test

```sh
cargo test --locked
```

## Deploy

```sh
aomi-build login --backend https://api-staging.aomi.dev
aomi-build project create
aomi-build deploy
```

`aomi.toml` leaves `server_tags` unset, so only staging backends load the
release. Set `server_tags = ["prod"]` and deploy against
`https://api.aomi.dev` for production.

## Callers

- `web/` renders `<AomiWidget>` in direct mode for the application id. Copy
  `web/src/main.tsx` into a site to embed the chat.
- `client/` sends one prompt from Node: `pnpm --filter client ask "Run strategy ag1ea6b8de"`.

Bump `BUILD` in `src/lib.rs` on every change; both tools report it as `app_build`.
