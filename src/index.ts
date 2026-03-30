export interface Env {
  SHARES: KVNamespace;
  ASSETS: Fetcher;
  SITE_NAME: string;
  OG_TITLE: string;
  OG_DESCRIPTION: string;
  OG_IMAGE: string;
  THEME_COLOR: string;
}

const PFEIFFER = '#8B3FA6';
const BCI_BLUE = '#3B82F6';
const BCI_RED = '#EF4444';

function ogHTML(env: Env, path: string, title?: string, description?: string, image?: string): string {
  const t = title || env.OG_TITLE;
  const d = description || env.OG_DESCRIPTION;
  const img = image || env.OG_IMAGE;
  const url = `https://${env.SITE_NAME}${path}`;

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${t}</title>

<!-- Open Graph -->
<meta property="og:type" content="website">
<meta property="og:url" content="${url}">
<meta property="og:title" content="${t}">
<meta property="og:description" content="${d}">
<meta property="og:image" content="${img}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:site_name" content="${env.SITE_NAME}">

<!-- Twitter Card -->
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="${t}">
<meta name="twitter:description" content="${d}">
<meta name="twitter:image" content="${img}">

<!-- Theme -->
<meta name="theme-color" content="${env.THEME_COLOR}">
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>🧠</text></svg>">

<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    min-height: 100vh;
    display: flex; align-items: center; justify-content: center;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    background: linear-gradient(135deg, ${BCI_BLUE} 0%, ${PFEIFFER} 50%, ${BCI_RED} 100%);
    color: #fff;
  }
  main {
    max-width: 640px; padding: 3rem 2rem; text-align: center;
  }
  h1 { font-size: 2.5rem; font-weight: 800; margin-bottom: 1rem; letter-spacing: -0.02em; }
  p { font-size: 1.125rem; opacity: 0.9; line-height: 1.6; margin-bottom: 1.5rem; }
  .mono { font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.875rem; opacity: 0.7; }
  a { color: inherit; text-decoration: underline; text-underline-offset: 2px; }
</style>
</head>
<body>
<main>
  <h1>${t}</h1>
  <p>${d}</p>
  <p class="mono">${url}</p>
</main>
</body>
</html>`;
}

function ogImageSVG(title: string, description: string): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="${BCI_BLUE}"/>
      <stop offset="50%" stop-color="${PFEIFFER}"/>
      <stop offset="100%" stop-color="${BCI_RED}"/>
    </linearGradient>
  </defs>
  <rect width="1200" height="630" fill="url(#bg)"/>
  <text x="600" y="260" text-anchor="middle" font-family="-apple-system,BlinkMacSystemFont,sans-serif" font-size="72" font-weight="800" fill="white">${escXML(title)}</text>
  <text x="600" y="340" text-anchor="middle" font-family="-apple-system,BlinkMacSystemFont,sans-serif" font-size="28" fill="white" opacity="0.9">${escXML(description.substring(0, 80))}</text>
  <text x="600" y="540" text-anchor="middle" font-family="monospace" font-size="20" fill="white" opacity="0.5">bci.blue</text>
</svg>`;
}

function escXML(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function escHTML(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' },
  });
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    // CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type, Authorization',
        },
      });
    }

    // OG image generation
    if (path === '/og.png' || path === '/og.svg') {
      const title = url.searchParams.get('title') || env.OG_TITLE;
      const desc = url.searchParams.get('desc') || env.OG_DESCRIPTION;
      const svg = ogImageSVG(title, desc);
      return new Response(svg, {
        headers: { 'Content-Type': 'image/svg+xml', 'Cache-Control': 'public, max-age=3600' },
      });
    }

    // .well-known for universal links / app association
    if (path === '/.well-known/assetlinks.json') {
      return jsonResponse([]);
    }
    if (path === '/.well-known/apple-app-site-association') {
      return jsonResponse({ applinks: { apps: [], details: [] } });
    }

    // API: create a share
    if (path === '/api/share' && request.method === 'POST') {
      try {
        const body = await request.json() as Record<string, string>;
        const id = crypto.randomUUID().substring(0, 8);
        const entry = {
          id,
          title: body.title || '',
          description: body.description || '',
          url: body.url || '',
          image: body.image || '',
          created: new Date().toISOString(),
        };
        await env.SHARES.put(`share:${id}`, JSON.stringify(entry), { expirationTtl: 86400 * 365 });
        return jsonResponse({ ok: true, id, share_url: `https://${env.SITE_NAME}/s/${id}` }, 201);
      } catch {
        return jsonResponse({ error: 'invalid JSON body' }, 400);
      }
    }

    // API: list shares
    if (path === '/api/shares' && request.method === 'GET') {
      const list = await env.SHARES.list({ prefix: 'share:' });
      const shares = [];
      for (const key of list.keys) {
        const val = await env.SHARES.get(key.name);
        if (val) shares.push(JSON.parse(val));
      }
      return jsonResponse({ shares });
    }

    // Serve a specific share with OG tags
    if (path.startsWith('/s/')) {
      const id = path.substring(3);
      const raw = await env.SHARES.get(`share:${id}`);
      if (!raw) {
        return new Response(ogHTML(env, path, 'Not Found', 'This share does not exist.'), {
          status: 404,
          headers: { 'Content-Type': 'text/html; charset=utf-8' },
        });
      }
      const share = JSON.parse(raw) as Record<string, string>;
      return new Response(
        ogHTML(
          env,
          path,
          share.title || env.OG_TITLE,
          share.description || env.OG_DESCRIPTION,
          share.image || env.OG_IMAGE,
        ),
        { headers: { 'Content-Type': 'text/html; charset=utf-8' } },
      );
    }

    // bci.horse root → policy page
    const host = url.hostname;
    if (host === 'bci.horse' && (path === '/' || path === '')) {
      const policyUrl = new URL('/policy.html', request.url);
      return fetch(new Request(policyUrl.toString(), request));
    }

    // Fallback: serve OG landing page for unknown paths
    return new Response(ogHTML(env, path), {
      headers: { 'Content-Type': 'text/html; charset=utf-8' },
    });
  },
};
