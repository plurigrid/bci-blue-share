export interface Env {
  SHARES: KVNamespace;
}

interface Item {
  id: string;
  title: string;
  bin: 'red' | 'blue';
  source: string;
}

interface Agg {
  red: number;
  blue: number;
  abstain: number;
  total: number;
}

const HORSE_API = 'https://api.github.com/repos/plurigrid/horse/contents/trees';
const TREE_RE = /^bcf-(\d+)\.tree$/;
const TITLE_RE = /\\title\{([^}]+)\}/;

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'no-store',
    },
  });
}

async function fetchCatalog(env: Env): Promise<{ items: Item[]; fetched_at: string }> {
  const cached = await env.SHARES.get('catalog:v1');
  if (cached) return JSON.parse(cached);

  const list = await fetch(HORSE_API, {
    headers: {
      'User-Agent': 'bci-blue-share-worker',
      'Accept': 'application/vnd.github+json',
    },
  });
  if (!list.ok) {
    throw new Error(`horse list ${list.status}`);
  }
  const files = (await list.json()) as Array<{ name: string; download_url: string }>;
  const trees = files.filter((f) => TREE_RE.test(f.name));

  const items: Item[] = await Promise.all(
    trees.map(async (f) => {
      const m = f.name.match(TREE_RE)!;
      const id = `bcf-${m[1]}`;
      const num = parseInt(m[1], 10);
      const bin: 'red' | 'blue' = num % 2 === 0 ? 'blue' : 'red';
      let title = id;
      try {
        const tree = await (await fetch(f.download_url)).text();
        const tm = tree.match(TITLE_RE);
        if (tm && tm[1]) title = tm[1].trim();
      } catch {
        /* keep id as title */
      }
      return {
        id,
        title,
        bin,
        source: `https://github.com/plurigrid/horse/blob/main/trees/${f.name}`,
      };
    })
  );

  items.sort((a, b) => a.id.localeCompare(b.id));
  const payload = { items, fetched_at: new Date().toISOString() };
  await env.SHARES.put('catalog:v1', JSON.stringify(payload), { expirationTtl: 300 });
  return payload;
}

async function readAgg(env: Env, item: string): Promise<Agg> {
  const raw = await env.SHARES.get(`agg:${item}`);
  if (!raw) return { red: 0, blue: 0, abstain: 0, total: 0 };
  return JSON.parse(raw) as Agg;
}

async function writeVote(env: Env, item: string, choice: -1 | 0 | 1): Promise<Agg> {
  const agg = await readAgg(env, item);
  if (choice === -1) agg.red += 1;
  else if (choice === 1) agg.blue += 1;
  else agg.abstain += 1;
  agg.total = agg.red + agg.blue + agg.abstain;
  await env.SHARES.put(`agg:${item}`, JSON.stringify(agg));
  return agg;
}

async function listAggs(env: Env): Promise<Record<string, Agg>> {
  const list = await env.SHARES.list({ prefix: 'agg:' });
  const out: Record<string, Agg> = {};
  for (const k of list.keys) {
    const v = await env.SHARES.get(k.name);
    if (v) out[k.name.substring(4)] = JSON.parse(v) as Agg;
  }
  return out;
}

export async function handleApi(request: Request, env: Env): Promise<Response | null> {
  const url = new URL(request.url);
  const path = url.pathname;

  if (path === '/api/catalog' && request.method === 'GET') {
    try {
      const cat = await fetchCatalog(env);
      return jsonResponse(cat);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      return jsonResponse({ error: 'catalog failed', detail: msg }, 502);
    }
  }

  if (path === '/api/vote' && request.method === 'POST') {
    let body: unknown;
    try {
      body = await request.json();
    } catch {
      return jsonResponse({ error: 'invalid json' }, 400);
    }
    const b = body as { item?: unknown; choice?: unknown };
    const item = typeof b.item === 'string' ? b.item : '';
    const choiceRaw = typeof b.choice === 'number' ? b.choice : Number(b.choice);
    if (!/^bcf-\d+$/.test(item)) {
      return jsonResponse({ error: 'invalid item' }, 400);
    }
    if (choiceRaw !== -1 && choiceRaw !== 0 && choiceRaw !== 1) {
      return jsonResponse({ error: 'choice must be -1, 0, or 1' }, 400);
    }
    const agg = await writeVote(env, item, choiceRaw as -1 | 0 | 1);
    return jsonResponse({ ok: true, item, agg });
  }

  if (path === '/api/agg' && request.method === 'GET') {
    const aggregates = await listAggs(env);
    return jsonResponse({ aggregates });
  }

  const aggMatch = path.match(/^\/api\/agg\/(bcf-\d+)$/);
  if (aggMatch && request.method === 'GET') {
    const item = aggMatch[1];
    const agg = await readAgg(env, item);
    return jsonResponse({ item, agg });
  }

  return null;
}
