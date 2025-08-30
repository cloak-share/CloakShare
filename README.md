# CloakShare — Live Screen‑Share Redaction (MVP)

> **Share, without the overshare.** CloakShare opens a **Safe Mirror** window of your screen and **cloaks** sensitive regions (PII/secrets) in real time, so anything you share on Zoom/Meet/Teams is sanitized automatically.

**Codename:** `SafeShare` (used in the starter scaffold). You can keep SafeShare internally and brand externally as CloakShare.

---

## ✨ Goals (MVP)

- Prevent accidental leaks of **API keys, tokens, emails, credit cards, JWTs, `.env` values** during screen sharing.
- Work **across apps** (browser, terminal, Slack desktop, IDE, PDFs) — not just the browser DOM.
- Be **privacy‑first** (no‑log by default), **low‑latency**, and **easy to share** in any meeting app.

---

## ✅ MVP Scope (In vs Out)

**In (first 6–8 weeks):**
- **Safe Mirror** window (macOS first) that you share in Zoom/Meet/Teams.
- **Auto‑detection + solid‑box redaction** for: emails, credit cards (Luhn), JWTs, common key prefixes (AWS/GCP/Slack/GitHub), `.env` tokens.
- **Detection cadence:** OCR every **200–400 ms**, scene‑change aware.
- **GPU overlays** (solid box default; pixelation optional later).
- **Per‑app presets** (Chrome, Terminal, Slack, VS Code), **policy modes** (Strict/Balanced/Light), **do‑not‑blur lists**.
- **No‑log default** (counts‑only telemetry). Signed/notarized app & clean permissions UX.

**Out (post‑MVP):**
- Windows parity (DXGI/D3D11) and Linux.
- Encrypted retention + EU data residency.
- Admin console (org policies, SSO/MDM).
- Face/name anonymization; screenshot redactor utility.
- Virtual “Safe Display” driver for one‑click sharing.

---

## 📈 Performance & Accuracy Targets

- **Overlay latency**: P50 ≤ **120 ms**, P95 ≤ **250 ms** from “PII appears” → “box visible”.
- **FPS**: Safe Mirror **60 fps** (fallback 30 fps on low‑power).
- **CPU**: ≤ **25%** on M1/M2 under heavy scenes (typical 10–15%).
- **GPU**: ≤ **20%** of iGPU budget.
- **Accuracy on test corpus**: **FN < 1%**, **FP < 3%**.
- **Startup‑to‑ready**: ≤ **3 s** cold start; ≤ **1 s** when running.

---

## 🧱 Architecture (High Level)

```
Capture (ScreenCaptureKit) → GPU texture (Metal)
             │
             ├─► Detection loop (every 200–400 ms)
             │      ├─ Downscale + tile
             │      ├─ OCR (PP‑OCRv3 via ONNX Runtime)
             │      └─ PII/Secrets rules + heuristics → [PII mask list]
             │
Render loop 60fps (Metal):
Captured texture + draw solid boxes for current [PII mask list] → Safe Mirror window
```

**Privacy:** OCR text processed in RAM and immediately discarded. Telemetry stores **counts/latency only**, no content.

---

## 🛠️ Tech Stack

- **Core:** Rust or C++ (low latency, direct GPU interop)
- **UI Shell:** Tauri (Rust) for fast packaging & updates
- **Capture (macOS):** ScreenCaptureKit (`SCStream`) → Metal textures
- **Rendering:** Metal (shader compositor; solid boxes default)
- **OCR:** PP‑OCRv3 (PaddleOCR) via **ONNX Runtime** (CPU; consider CoreML later)
- **PII/Secrets rules:** regex + checksum (Luhn/IBAN) + token prefixes + entropy heuristic
- **Tracking:** IoU/optical flow & scene‑change diffing to avoid re‑OCR every frame
- **Packaging:** macOS hardened runtime + notarization; auto‑updates (Sparkle/Squirrel)
- **Telemetry:** counts & latency buckets only; crash reporting with hard PII scrubbing

---

## 🧩 Builder Plan — Step by Step (maps to your 5 steps)

### 1) Show the screen “as it is” (Safe Mirror)
**Goal:** Low‑latency mirrored window of a selected display/window.

**How (macOS):**
- Request **Screen Recording** permission.
- Use **ScreenCaptureKit** (`SCShareableContent`, `SCStream`, `SCStreamOutput`) to capture display/window.
- Convert `CMSampleBuffer` → **Metal texture**; draw to an NSWindow/`CAMetalLayer` (Tauri plugin).

**Accept:** Smooth **60 fps** at 1080p/1440p, near‑zero perceived latency, sharing the Safe Mirror looks identical to source.

**Gotchas:** Pin color‑space to **BGRA8**; avoid CPU copies; stay on GPU.

---

### 2) Static black box overlay (visible when sharing)
**Goal:** Confirm we can render overlays that viewers will see.

**How:** After drawing the captured texture, draw a **solid black rectangle** in the Metal pass; simple UI to toggle/drag it.

**Accept:** Box is always visible locally and to remote viewers; no frame drops.

**Gotcha:** Never draw into the source app — overlays live **only** in the Safe Mirror compositor.

---

### 3) Find PII on the screen (pixel feed)
**Goal:** Detect sensitive text regions with speed & reliability.

**Pipeline:**
1) **Tile & downscale** the frame (e.g., to 0.5×, tiles ~640×360).
2) **OCR** (PP‑OCRv3 via ONNX) only on tiles with text‑like signal.
3) **Rules:**
   - **Emails** (RFC‑lite), **phones**, **credit cards** (13–19 digits + **Luhn**).
   - **API keys** (prefixes: `AKIA`, `ASIA`, `ghp_`, `xoxp-`, `xoxb-`, `AIza`, `sk-`, `-----BEGIN PRIVATE KEY-----`).
   - **JWT** (`xxxxx.yyyyy.zzzzz` Base64URL).
   - **.env tokens**: `NAME=VALUE` with high‑entropy values.
   - Optional **entropy** check to confirm “secret‑ness”.
4) **UI heuristics**: downweight code blocks/monospace; ignore Figma/design canvases; boost known input fields (Password/Email).
5) **Confidence/box padding**: score > threshold (e.g., 0.8); pad 3–6 px for OCR jitter.

**Test corpus:** Screens with fake CCs, emails, tokens, JWTs, `.env`, terminal logs, plus non‑PII dense UIs to measure FPs.

**Accept:** On corpus, **FN < 1%**, **FP < 3%**; pass ≤ **250 ms** at 1440p on M1.

---

### 4) Do we scan every frame?
**Short answer:** **No** — scan **periodically** and on **scene change**; render masks **every frame**.

**Strategy:**
- Render loop at **60 fps** always draws last known masks.
- OCR + rules cadence **200–400 ms**.
- **Scene‑change trigger:** per‑tile grayscale histogram diff or SSIM‑lite; only changed tiles enter OCR next pass.
- **Region tracking:** IoU/optical flow to follow boxes while scrolling.
- **Backoff:** if static for N seconds, slow checks; if big diff (scroll/type), speed up to 200 ms.

**Accept:** CPU ≤ **25%** in busy UIs; overlays stick while scrolling/resizing.

---

### 5) Update with black boxes around PII
**Goal:** Stable, non‑flickery overlays that users trust.

**How:**
- Maintain a **PII mask list**: `{ bbox, type, confidence, last_seen_ts }`.
- Each frame: drop stale masks (>1.5 s unseen), **smooth** edges (avg last 2–3 positions), draw **solid boxes** (default). Pixelation/blur optional.
- Add **local‑only labels** (“CARD”, “KEY”) for debug; never stream labels unless configured.
- Optional **local uncloak** hotkey (owner view only) for verification; **never** uncloak in the shared stream.

**Accept:** No visible flicker; boxes track within a frame or two; disappear cleanly when content is gone.

---

## 🖥️ Minimal UI (MVP)

- Big toggle: **Safe Share ON/OFF**
- Button: **Pick display/window**
- Mode: **Strict / Balanced / Light** (thresholds under‑the‑hood)
- **App presets:** Chrome, Terminal, Slack, VS Code (checkboxes)
- **Hotkeys:** toggle overlay, show debug

---

## 📊 Metrics to Track

- **Overlay latency**: detection timestamp → first masked frame (**P95 ≤ 250 ms**).
- **CPU/GPU** under scrolling/typing (CPU ≤ 25%).  
- **False pos/neg** on test corpus (CI gate).  
- **Frame pacing**: missed frames/min at 60 fps (negligible).

---

## 🚀 “Hello World” MVP Timeline (2–3 weeks)

**Week 1**
- Capture → Metal render (Step 1)
- Static black box overlay + hotkeys (Step 2)

**Week 2**
- Tile + OCR + rules (Step 3)
- Draw boxes from detected bboxes (Step 5 basic)
- Share Safe Mirror into Zoom/Meet sanity check

**Week 3**
- Scene‑change diff + cadence (Step 4)
- Region tracking + jitter smoothing (Step 5 polish)
- Tiny test corpus + perf logs; tighten thresholds
- Ship MVP → **2–3 real pilots**

---

## 🧪 Test Corpus (how to build it fast)

- **PII set:** generate fake emails/phones; credit card patterns with **Luhn‑valid** numbers (test ranges); JWTs with dummy payload; `.env` files with random high‑entropy values; AWS/GCP/GitHub/Slack key‑like strings.
- **Non‑PII set:** dense dashboards, Figma/Canva canvases, docs, code blocks.
- Save as PNGs at 1080p/1440p; script expected boxes for automated FN/FP scoring.

---

## 🔐 Privacy Model

- **No‑log by default:** we do **not store content** (originals or translations). OCR text exists only in RAM and is discarded after classification.
- **Telemetry:** counts & latency buckets only (no strings). Crash logs scrub PII.
- **Future (opt‑in):** encrypted retention with per‑tenant keys, TTL purge, export/delete endpoints; **EU region** data residency.

---

## 📦 Starter Scaffold (included)

A minimal Rust + Tauri app is provided to get pixels on screen **today** (CoreGraphics snapshots → canvas) and draw a **static black box**. This is **temporary**; swap to **ScreenCaptureKit + Metal** for the real pipeline.

**Run (macOS):**
```bash
# prerequisites: Rust stable, Node LTS, Xcode CLT, `cargo install tauri-cli`
unzip safeshare-starter.zip
cd safeshare-starter
npm install
npm run tauri   # grant Screen Recording when prompted
# Share the “SafeShare — MVP” window in Zoom/Meet to verify overlays are visible
```

**Next commits:**
- [ ] Replace snapshots with **ScreenCaptureKit** streamer + **Metal** compositor
- [ ] 60 fps frame pacing & timing
- [ ] ONNX Runtime + **PP‑OCRv3** integration
- [ ] PII/Secrets rules (email/phone/CC Luhn/JWT/AWS+GitHub+Slack/`.env`) + entropy heuristic
- [ ] Scene‑change detector (tile histogram diff) @200–400 ms
- [ ] Region tracker (IoU/flow) + jitter smoothing
- [ ] Solid‑box compositor default; pixelation optional
- [ ] “Share the **Safe Mirror** window” banner
- [ ] Perf counters (CPU, detection ms, overlay latency) & CI corpus gate

---

## 📜 License

- Scaffold: MIT (adjust for product needs).  
- Make sure any OCR/model licenses fit your distribution (PP‑OCRv3 & ONNX Runtime are permissive).

---

## 🧭 Naming & Positioning

- **Product name:** **CloakShare** (brand)  
- **Codename:** **SafeShare** (repo/package)  
- **Tagline:** *Share, without the overshare.*  
- **Default policy:** **solid‑box redaction** (anti‑deblur); blur/pixelation optional.

---

Questions or contributions? Open an issue titled **[MVP]** with steps to reproduce and your device specs (macOS version, CPU/GPU).
