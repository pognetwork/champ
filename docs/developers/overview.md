# Overview

We've created a diagram to help understand the general structure of this project:

[![techstack](./techstack.drawio.svg)](./techstack.drawio.svg)

## gRPC API

Our gRPC API is one of the main ways external applications interact with our software. gRPC is a transport protocol based on HTTP, enabling us to do typesafe procedure calls through a nice API. In addition, we've enabled `grpc-web` support which allows web applications to interact with this API, similarly to how you would call a standard web API.

For further information, check out our [API Documentation](./rpc-api.md) with some more documentation on the purpose of all of our methods.
