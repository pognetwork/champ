# Security at Pog.Network

## Threat Model
![](threat.drawio.svg)

## Threats

- ### All outgoing connections
    > DDOS: We plan to add rate limiting and we will guide Node Operators for optimal setup as we won't run the nodes ourselves.
    <br>(_Denial of Service, High Severity_)

    > Insecure SSL version: In release we plan to refuse connections with a TLS version different than 1.3 and refuse insecure connections. Currently, everything is accepted for development purposes.
    <br> (_Tamperling, High Severity_)
    
- ### Web Clients
    > XSS Attacks: `Mitigated` because React sanitizes all user input and therefore prevents accidental script injections. Furthermore, we plan to implement a Content Security Policy.
    <br> (_Elevation of Privilage, High Severity_)

- ### Node Admin and Node Wallet Manager only
    > CSRF Attacks: `Mitigated` because we avoid using Cookies that could carry sensitive information. Our short lived JWTs are stored in the browser session storage.
    <br> (_Spoofing, High Severity_)

- ### Node Admin Service and Node Wallet Manager Service
    **Description:**
    These Services are the endpoints for the Web Clients of similar name.
    The severtiy of these attacks are lower, as these enpoints are not open to the internet by default but are contained within the local network.

    > Credential Stuffing: `Mitigated` because we hash passwords in storage and we plan to use the [Have I Been Pwned](https://haveibeenpwned.com/) API to securly check if a password has been part of a breach. 
    <br> (_Elevation of Privilage, Medium Severity_)

    > Credential Cracking: `Mitigated` because we hash passwords that take longer to compare and we plan to implement a rate limiting mechanism.
    <br> (_Elevation of Privilage, High Severity_)

- ### Block Service
    **Description:**
    This is the endpoint to the Wallet Web Client that handels user requests to view their wallet or chain information.

    > High Complexity Queries: We plan to add size limits to requests to preventing users from executing high workload requests.
    <br> (_Denial of Service, Medium Severity_)

- ### Node
    > DDOS: We plan to add rate limiting and we will guide Node Operators for optimal setup as we won't run the nodes ourselves.
    <br>(_Denial of Service, High Severity_)

- ### Peer to Peer
    > DDOS: We plan to add rate limiting and we will guide Node Operators for optimal setup as we won't run the nodes ourselves.
    <br>(_Denial of Service, High Severity_)