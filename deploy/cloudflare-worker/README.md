# NetFreedom Cloudflare Worker (VLESS over WebSocket)

This directory contains the production-ready Cloudflare Worker script that powers the **NetFreedom Stealth Tunnel (TCP Port 443)**.

---

## Why is This Needed?

Enterprise firewalls (such as Fortinet, Palo Alto, and Cisco) in university campuses and corporate networks often enforce:
1. **Outbound UDP Blackholing:** Dropping all standard WireGuard (`162.159.192.1:2408`) and OpenVPN traffic.
2. **Deep Packet Inspection (DPI) & Full TCP Stream Reassembly:** When stateful firewalls reassemble fragmented TCP segments, packet desynchronization tools (ByeDPI / GoodByeDPI) cannot fool the middlebox because the firewall inspects the fully reconstructed TLS ClientHello before forwarding it.

By routing your traffic through this Cloudflare Worker:
- Traffic looks 100% identical to standard, legitimate corporate web traffic (`wss://your-worker.workers.dev:443`).
- Fortinet sees only a standard Google Chrome TLS 1.3 handshake to Cloudflare's global CDN Anycast IP.
- The firewall cannot inspect inside the encrypted WebSocket tunnel.

---

## 60-Second Setup Guide (100% Free)

### Step 1: Create a Free Cloudflare Account
If you don't have one, sign up at [dash.cloudflare.com](https://dash.cloudflare.com) (free forever tier, 100,000 requests/day).

### Step 2: Create a Worker
1. In the Cloudflare Dashboard, go to **Compute (Workers) > Workers & Pages**.
2. Click **Create Application** > **Create Worker**.
3. Name your worker (e.g., `netfreedom-tunnel`) and click **Deploy**.

### Step 3: Paste Worker Code
1. Click **Edit code** in the top right.
2. Replace all the code in `worker.js` with the contents of [`vless-worker.js`](./vless-worker.js).
3. (Optional) Replace the `DEFAULT_UUID` with your own UUID (generate one on [uuidgenerator.net](https://www.uuidgenerator.net/)), or configure the `UUID` environment variable in Settings > Variables.
4. Click **Deploy** (or **Save and Deploy**).

### Step 4: Configure NetFreedom
1. Open **NetFreedom Desktop**.
2. Go to **Profiles** (or **Settings**).
3. On the **Campus / Enterprise Wi-Fi Stealth (TCP 443)** card, click **Configure Node**.
4. Enter:
   - **Worker Host:** `netfreedom-tunnel.<your-subdomain>.workers.dev`
   - **UUID:** Your configured UUID (default: `d342d110-d4a3-45c3-b0a3-3b11815b80b2`)
   - **CDN Anycast IP:** Leave blank or use `104.16.132.229` (default)
5. Click **Save Configuration**.
6. Switch to **Stealth Tunnel** and click **Connect**.
