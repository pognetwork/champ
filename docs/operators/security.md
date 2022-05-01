# Security considerations when running a pog.network node

Once the full release of `champ` is ready, more information will follow.

- **Firewall**<br/>
  If you're not 100% sure you need to, don't expose the gRPC API to the public internet.
- **Tunneling**<br/>
  For large deployments, we recommend not exposing your node directly to the internet instead of using a reverse proxy on a separate server, for example, using [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/tutorials/warp-to-tunnel).
