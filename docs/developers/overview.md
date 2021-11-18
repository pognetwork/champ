# Overview

We've created a diagram to help understand the general structure of this project:

[![techstack](./techstack.drawio.svg)](./techstack.drawio.svg)

## gRPC API

Our gRPC API is one of the main ways for external applications to interact with our software. gRPC is a transport protocol based on HTTP which enables us to do typesafe procedure calls through a nice api. In addition, we've enables `grpc-web` support which enables web applications to interact with this api, similarly to how you would call a normal web api.

For further information, check out our [API Documentation](./rpc-api.md) with some more documentation on the purpose of all of our methods.
