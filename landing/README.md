---
home: true
heroImage: /logo.svg
heroText: NeonD
tagline: DX-focused control plane for PostgreSQL — branching, point-in-time recovery and S3 durability in a single container.
actions:
  - text: Get started
    link: /docs/
    type: primary
  - text: GitHub
    link: https://github.com/matisiekpl/neond
    type: secondary
features:
  - title: Instant branching
    details: Branch your production timeline to development or preview branches in seconds, with precise point-in-time recovery.
  - title: S3 durability
    details: Layer files and management-DB backups stream to Amazon S3. Wipe local, restore on boot, move servers with a copy.
  - title: PgBouncer built-in
    details: Connection pooling out of the box on every endpoint. No separate sidecar to run.
  - title: Point-in-time recovery
    details: Restore any branch — or fork a new one — from a precise moment in the past. No backup juggling.
  - title: TLS SNI routing
    details: Generates {instance}.your-company.com endpoints so every Postgres instance lives under a single domain.
  - title: Single-container deploy
    details: One image, one data directory, one lockfile. Designed for small VPS servers and early-stage teams.
footer: Apache 2.0 Licensed | Copyright © Mateusz Woźniak
---

<div class="screenshot">

![NeonD dashboard](/screenshot.png)

</div>

<style>
.vp-home {
  display: flex;
  flex-direction: column;
}
.vp-home > .vp-hero { order: 1; }
.vp-home > [vp-content] { order: 2; }
.vp-home > .vp-features { order: 3; }
.vp-home > .vp-footer { order: 4; }

.screenshot {
  max-width: 1100px;
  margin: 0 auto 2rem;
  padding: 0 1rem;
}
.screenshot img {
  width: 100%;
  height: auto;
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
}
</style>

