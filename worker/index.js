export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (!url.pathname.endsWith(".wasm")) {
      return env.ASSETS.fetch(request);
    }

    // The asset contains pre-compressed Brotli bytes. Fetch the stored bytes
    // without content negotiation, then tell the client to decode them once.
    const assetRequest = new Request(request);
    assetRequest.headers.set("Accept-Encoding", "identity");
    const assetResponse = await env.ASSETS.fetch(assetRequest);

    if (!assetResponse.ok) {
      return assetResponse;
    }

    const headers = new Headers(assetResponse.headers);
    headers.set("Content-Encoding", "br");
    headers.set("Content-Type", "application/wasm");
    headers.set("Cache-Control", "public, max-age=31536000, immutable");
    headers.set("Cloudflare-CDN-Cache-Control", "public, max-age=31536000");

    return new Response(assetResponse.body, {
      status: assetResponse.status,
      statusText: assetResponse.statusText,
      headers,
      encodeBody: "manual",
    });
  },
};
