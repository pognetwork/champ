# Security considerations when running a pog.network node

More information will follow once the full release of `champ` is ready.

- **Firewall**<br/>
  If you're not 100% sure you need to, don't expose the gRPC API to the public internet.
- **Tunneling**<br/>
  For large deployments, it's recommended to not expose your node directly to the internet, and instead use a reverse proxy on a separate server, for example using [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/tutorials/warp-to-tunnel).
