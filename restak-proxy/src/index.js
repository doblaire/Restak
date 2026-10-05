// Cloudflare Worker: Secure SteamGridDB Poster Proxy & KV Global Database for Restak
// 1. Encrypts and protects the SteamGridDB API key (never exposed in client).
// 2. Persistent Global Database via Cloudflare KV: Once any user searches for a game,
//    its 600x900 posters are stored in Cloudflare KV permanently.
//    Subsequent requests by ANY user pull directly from Cloudflare KV, consuming ZERO SteamGridDB API calls.
// 3. Cloudflare Edge Caching (caches.default) for sub-millisecond edge latency.
// 4. Full CORS support for cross-origin requests.

export default {
  async fetch(request, env, ctx) {
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, {
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type, User-Agent, X-Restak-Client",
        },
      });
    }

    if (request.method !== "GET") {
      return new Response(JSON.stringify({ error: "Method not allowed" }), {
        status: 405,
        headers: { "Content-Type": "application/json" },
      });
    }

    const url = new URL(request.url);
    const title = url.searchParams.get("title");

    if (!title || !title.trim()) {
      return new Response(JSON.stringify({ error: "Missing 'title' parameter" }), {
        status: 400,
        headers: { "Content-Type": "application/json" },
      });
    }

    const apiKey = env.STEAMGRIDDB_API_KEY;
    if (!apiKey) {
      return new Response(JSON.stringify({ error: "Server API key configuration missing" }), {
        status: 500,
        headers: { "Content-Type": "application/json" },
      });
    }

    // Clean title query: remove extension, bracketed tags, region indicators, etc.
    const cleanTitle = title
      .replace(/\.[a-zA-Z0-9]+$/g, "")
      .replace(/\(.*?\)/g, "")
      .replace(/\[.*?\]/g, "")
      .replace(/Rev\s+\d+/gi, "")
      .replace(/USA|Europe|Japan/gi, "")
      .trim();

    const normalizedKey = cleanTitle.toLowerCase();
    const kvKey = `cover:${normalizedKey}`;

    // 1. Check Cloudflare KV Persistent Global Database first
    if (env.RESTAK_COVERS_KV) {
      try {
        const kvRecord = await env.RESTAK_COVERS_KV.get(kvKey, { type: "json" });
        if (kvRecord && kvRecord.covers && kvRecord.covers.length > 0) {
          return new Response(JSON.stringify(kvRecord), {
            headers: {
              "Content-Type": "application/json",
              "Access-Control-Allow-Origin": "*",
              "Cache-Control": "public, max-age=86400, s-maxage=604800",
              "X-Restak-Cache": "KV-HIT",
            },
          });
        }
      } catch (err) {
        console.error("KV read error:", err);
      }
    }

    // 2. Check Cloudflare Edge Cache
    const cacheUrl = new URL(request.url);
    cacheUrl.search = `?q=${encodeURIComponent(normalizedKey)}`;
    const cacheKey = new Request(cacheUrl.toString(), request);
    const cache = caches.default;

    let cachedResponse = await cache.match(cacheKey);
    if (cachedResponse) {
      return cachedResponse;
    }

    try {
      // 3. Query SteamGridDB for the game ID
      const searchUrl = `https://www.steamgriddb.com/api/v2/search/autocomplete/${encodeURIComponent(cleanTitle)}`;
      const searchRes = await fetch(searchUrl, {
        headers: {
          Authorization: `Bearer ${apiKey}`,
        },
      });

      if (!searchRes.ok) {
        return new Response(JSON.stringify({ success: false, covers: [] }), {
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
          },
        });
      }

      const searchData = await searchRes.json();
      if (!searchData.success || !searchData.data || searchData.data.length === 0) {
        return new Response(JSON.stringify({ success: true, covers: [] }), {
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
          },
        });
      }

      // Collect top game IDs
      const gameIds = searchData.data.slice(0, 3).map((g) => g.id);
      const allCovers = [];
      const seenUrls = new Set();

      // 4. Fetch 600x900 vertical posters for matching games
      for (const gameId of gameIds) {
        const gridUrl = `https://www.steamgriddb.com/api/v2/grids/game/${gameId}?dimensions=600x900`;
        const gridRes = await fetch(gridUrl, {
          headers: {
            Authorization: `Bearer ${apiKey}`,
          },
        });

        if (gridRes.ok) {
          const gridData = await gridRes.json();
          if (gridData.success && gridData.data) {
            for (const item of gridData.data) {
              const coverUrl = item.url || item.thumb;
              if (coverUrl && !seenUrls.has(coverUrl)) {
                seenUrls.add(coverUrl);
                allCovers.push(coverUrl);
              }
            }
          }
        }
      }

      const resultPayload = {
        success: true,
        game_name: searchData.data[0].name,
        covers: allCovers,
      };

      // 5. Persist to Cloudflare KV database permanently
      if (env.RESTAK_COVERS_KV && allCovers.length > 0) {
        try {
          await env.RESTAK_COVERS_KV.put(kvKey, JSON.stringify(resultPayload));
        } catch (e) {
          console.error("KV put error:", e);
        }
      }

      const finalResponse = new Response(JSON.stringify(resultPayload), {
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
          "Cache-Control": "public, max-age=86400, s-maxage=604800",
          "X-Restak-Cache": "STEAMGRIDDB-FETCH",
        },
      });

      // Cache at the Cloudflare Edge
      ctx.waitUntil(cache.put(cacheKey, finalResponse.clone()));

      return finalResponse;
    } catch (err) {
      return new Response(JSON.stringify({ error: err.message, covers: [] }), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
        },
      });
    }
  },
};
